use crate::core::sync::{Backoff, CachePadded};
use alloc::{
    alloc::{Layout, alloc_zeroed, handle_alloc_error},
    boxed::Box,
};
use core::{
    cell::UnsafeCell,
    marker::PhantomData,
    mem::MaybeUninit,
    ptr,
    sync::atomic::{self, AtomicPtr, AtomicUsize, Ordering},
};

#[cfg(target_has_atomic = "64")]
type AtomicIndex = atomic::AtomicU64;
#[cfg(target_has_atomic = "64")]
type Index = u64;
#[cfg(not(target_has_atomic = "64"))]
type AtomicIndex = atomic::AtomicUsize;
#[cfg(not(target_has_atomic = "64"))]
type Index = usize;

// Bits indicating the state of a slot:
// * If a value has been written into the slot, `WRITE` is set.
// * If a value has been read from the slot, `READ` is set.
// * If the block is being destroyed, `DESTROY` is set.
const WRITE: usize = 1;
const READ: usize = 2;
const DESTROY: usize = 4;

// Each block covers one "lap" of indices.
const LAP: Index = 32;
// The maximum number of values a block can hold.
const BLOCK_CAP: usize = LAP as usize - 1;
// How many lower bits are reserved for metadata.
const SHIFT: usize = 1;
// Indicates that the block is not the last one.
const HAS_NEXT: Index = 1;

/// An unbounded multi-producer multi-consumer queue.
///
/// This queue is implemented as a linked list of segments, where each segment is a small buffer
/// that can hold a handful of elements. There is no limit to how many elements can be in the queue
/// at a time. However, since segments need to be dynamically allocated as elements get pushed,
/// this queue is somewhat slower than [`ArrayQueue`].
pub struct SegQueue<T> {
    head: CachePadded<Position<T>>,
    tail: CachePadded<Position<T>>,
    _marker: PhantomData<T>,
}

impl<T> SegQueue<T> {
    /// Creates a new unbounded queue.
    pub const fn new() -> SegQueue<T> {
        SegQueue {
            head: CachePadded::new(Position {
                block: AtomicPtr::new(ptr::null_mut()),
                index: AtomicIndex::new(0),
            }),
            tail: CachePadded::new(Position {
                block: AtomicPtr::new(ptr::null_mut()),
                index: AtomicIndex::new(0),
            }),
            _marker: PhantomData,
        }
    }

    /// Returns `true` if the queue is empty.
    pub fn is_empty(&self) -> bool {
        let head = self.head.index.load(Ordering::SeqCst);
        let tail = self.tail.index.load(Ordering::SeqCst);
        head >> SHIFT == tail >> SHIFT
    }

    /// Returns the number of elements in the queue.
    pub fn len(&self) -> usize {
        loop {
            // Load the tail index, then load the head index.
            let mut tail = self.tail.index.load(Ordering::SeqCst);
            let mut head = self.head.index.load(Ordering::SeqCst);

            // If the tail index didn't change, we've got consistent indices to work with.
            if self.tail.index.load(Ordering::SeqCst) == tail {
                // Erase the lower bits.
                tail &= !((1 << SHIFT) - 1);
                head &= !((1 << SHIFT) - 1);

                // Fix up indices if they fall onto block ends.
                if (tail >> SHIFT) & (LAP - 1) == LAP - 1 {
                    tail = tail.wrapping_add(1 << SHIFT);
                }
                if (head >> SHIFT) & (LAP - 1) == LAP - 1 {
                    head = head.wrapping_add(1 << SHIFT);
                }

                // Rotate indices so that head falls into the first block.
                let lap = (head >> SHIFT) / LAP;
                tail = tail.wrapping_sub((lap * LAP) << SHIFT);
                head = head.wrapping_sub((lap * LAP) << SHIFT);

                // Remove the lower bits.
                tail >>= SHIFT;
                head >>= SHIFT;

                // Return the difference minus the number of blocks between tail and head.
                return (tail - head - tail / LAP) as usize;
            }
        }
    }

    /// Pushes back an element to the tail.
    pub fn push(&self, value: T) {
        let backoff = Backoff::new();
        let mut tail = self.tail.index.load(Ordering::Acquire);
        let mut block = self.tail.block.load(Ordering::Acquire);
        let mut next_block = None;

        loop {
            // Calculate the offset of the index into the block.
            let offset = ((tail >> SHIFT) % LAP) as usize;

            // If we reached the end of the block, wait until the next one is installed.
            if offset == BLOCK_CAP {
                backoff.snooze();
                tail = self.tail.index.load(Ordering::Acquire);
                block = self.tail.block.load(Ordering::Acquire);
                continue;
            }

            // If we're going to have to install the next block, allocate it in advance in order to
            // make the wait for other threads as short as possible.
            if offset + 1 == BLOCK_CAP && next_block.is_none() {
                next_block = Some(Block::<T>::new());
            }

            // If this is the first push operation, we need to allocate the first block.
            if block.is_null() {
                let new = Box::into_raw(Block::<T>::new());

                if self
                    .tail
                    .block
                    .compare_exchange(block, new, Ordering::Release, Ordering::Relaxed)
                    .is_ok()
                {
                    self.head.block.store(new, Ordering::Release);
                    block = new;
                } else {
                    next_block = unsafe { Some(Box::from_raw(new)) };
                    tail = self.tail.index.load(Ordering::Acquire);
                    block = self.tail.block.load(Ordering::Acquire);
                    continue;
                }
            }

            let new_tail = tail + (1 << SHIFT);

            // Try advancing the tail forward.
            match self.tail.index.compare_exchange_weak(
                tail,
                new_tail,
                Ordering::SeqCst,
                Ordering::Acquire,
            ) {
                Ok(_) => unsafe {
                    // If we've reached the end of the block, install the next one.
                    if offset + 1 == BLOCK_CAP {
                        let next_block = Box::into_raw(next_block.unwrap());
                        let next_index = new_tail.wrapping_add(1 << SHIFT);

                        self.tail.block.store(next_block, Ordering::Release);
                        self.tail.index.store(next_index, Ordering::Release);
                        (*block).next.store(next_block, Ordering::Release);
                    }

                    // Write the value into the slot.
                    let slot = (*block).slots.get_unchecked(offset);
                    slot.value.get().write(MaybeUninit::new(value));
                    slot.state.fetch_or(WRITE, Ordering::Release);

                    return;
                },
                Err(t) => {
                    tail = t;
                    block = self.tail.block.load(Ordering::Acquire);
                    backoff.spin();
                }
            }
        }
    }

    /// Pushes an element to the queue with exclusive mutable access.
    ///
    /// Avoids atomic operations and synchronization, assuming
    /// no other threads access the queue concurrently.
    pub fn push_mut(&mut self, value: T) {
        let tail = *self.tail.index.get_mut();
        let mut block = *self.tail.block.get_mut();

        // Calculate the offset of the index into the block.
        let offset = ((tail >> SHIFT) % LAP) as usize;

        // If this is the first push operation, we need to allocate the first block.
        if block.is_null() {
            let new = Box::into_raw(Block::<T>::new());
            *self.head.block.get_mut() = new;
            *self.tail.block.get_mut() = new;

            block = new;
        }

        let new_tail = tail + (1 << SHIFT);

        *self.tail.index.get_mut() = new_tail;

        unsafe {
            // If we've reached the end of the block, install the next one.
            if offset + 1 == BLOCK_CAP {
                let next_block = Box::into_raw(Block::<T>::new());
                let next_index = new_tail.wrapping_add(1 << SHIFT);

                *self.tail.block.get_mut() = next_block;
                *self.tail.index.get_mut() = next_index;
                *(*block).next.get_mut() = next_block;
            }

            // Write the value into the slot.
            let slot = (*block).slots.get_unchecked(offset);
            slot.value.get().write(MaybeUninit::new(value));
            *(*block).slots.get_unchecked_mut(offset).state.get_mut() |= WRITE;
        }
    }

    /// Pops the head element from the queue.
    pub fn pop(&self) -> Option<T> {
        let backoff = Backoff::new();
        let mut head = self.head.index.load(Ordering::Acquire);
        let mut block = self.head.block.load(Ordering::Acquire);

        loop {
            // Calculate the offset of the index into the block.
            let offset = ((head >> SHIFT) % LAP) as usize;

            // If we reached the end of the block, wait until the next one is installed.
            if offset == BLOCK_CAP {
                backoff.snooze();
                head = self.head.index.load(Ordering::Acquire);
                block = self.head.block.load(Ordering::Acquire);
                continue;
            }

            let mut new_head = head + (1 << SHIFT);

            if new_head & HAS_NEXT == 0 {
                atomic::fence(Ordering::SeqCst);
                let tail = self.tail.index.load(Ordering::Relaxed);

                // If the tail equals the head, that means the queue is empty.
                if head >> SHIFT == tail >> SHIFT {
                    return None;
                }

                // If head and tail are not in the same block, set `HAS_NEXT` in head.
                if (head >> SHIFT) / LAP != (tail >> SHIFT) / LAP {
                    new_head |= HAS_NEXT;
                }
            }

            // The block can be null here only if the first push operation is in progress. In that
            // case, just wait until it gets initialized.
            if block.is_null() {
                backoff.snooze();
                head = self.head.index.load(Ordering::Acquire);
                block = self.head.block.load(Ordering::Acquire);
                continue;
            }

            // Try moving the head index forward.
            match self.head.index.compare_exchange_weak(
                head,
                new_head,
                Ordering::SeqCst,
                Ordering::Acquire,
            ) {
                Ok(_) => unsafe {
                    // If we've reached the end of the block, move to the next one.
                    if offset + 1 == BLOCK_CAP {
                        let next = (*block).wait_next();
                        let mut next_index = (new_head & !HAS_NEXT).wrapping_add(1 << SHIFT);
                        if !(*next).next.load(Ordering::Relaxed).is_null() {
                            next_index |= HAS_NEXT;
                        }

                        self.head.block.store(next, Ordering::Release);
                        self.head.index.store(next_index, Ordering::Release);
                    }

                    // Read the value.
                    let slot = (*block).slots.get_unchecked(offset);
                    slot.wait_write();
                    let value = slot.value.get().read().assume_init();

                    // Destroy the block if we've reached the end, or if another thread wanted to
                    // destroy but couldn't because we were busy reading from the slot.
                    if offset + 1 == BLOCK_CAP {
                        Block::destroy(block, 0);
                    } else if slot.state.fetch_or(READ, Ordering::AcqRel) & DESTROY != 0 {
                        Block::destroy(block, offset + 1);
                    }

                    return Some(value);
                },
                Err(h) => {
                    head = h;
                    block = self.head.block.load(Ordering::Acquire);
                    backoff.spin();
                }
            }
        }
    }

    /// Pops the head element from the queue using an exclusive reference.
    ///
    /// Avoids atomic operations and synchronization, assuming
    /// no other threads access the queue concurrently.
    ///
    /// If the queue is empty, `None` is returned.
    pub fn pop_mut(&mut self) -> Option<T> {
        let head = *self.head.index.get_mut();
        let block = *self.head.block.get_mut();

        // Calculate the offset of the index into the block.
        let offset = ((head >> SHIFT) % LAP) as usize;

        let mut new_head = head + (1 << SHIFT);

        if new_head & HAS_NEXT == 0 {
            let tail = *self.tail.index.get_mut();

            // If the tail equals the head, that means the queue is empty.
            if head >> SHIFT == tail >> SHIFT {
                return None;
            }

            // If head and tail are not in the same block, set `HAS_NEXT` in head.
            if (head >> SHIFT) / LAP != (tail >> SHIFT) / LAP {
                new_head |= HAS_NEXT;
            }
        }

        *self.head.index.get_mut() = new_head;

        unsafe {
            // If we've reached the end of the block, move to the next one.
            if offset + 1 == BLOCK_CAP {
                let next = *(*block).next.get_mut();
                let mut next_index = (new_head & !HAS_NEXT).wrapping_add(1 << SHIFT);
                if !(*next).next.get_mut().is_null() {
                    next_index |= HAS_NEXT;
                }

                *self.head.block.get_mut() = next;
                *self.head.index.get_mut() = next_index;
            }

            // Read the value.
            let slot = (*block).slots.get_unchecked(offset);
            let value = slot.value.get().read().assume_init();

            // Destroy the block if we've reached the end
            if offset + 1 == BLOCK_CAP {
                Block::destroy_mut(block);
            } else {
                let state = *(*block).slots.get_unchecked_mut(offset).state.get_mut();
                *(*block).slots.get_unchecked_mut(offset).state.get_mut() = state | READ;
                if state & DESTROY != 0 {
                    Block::destroy(block, offset + 1);
                }
            }

            Some(value)
        }
    }
}

impl<T> Default for SegQueue<T> {
    fn default() -> SegQueue<T> {
        SegQueue::new()
    }
}

impl<T> Drop for SegQueue<T> {
    fn drop(&mut self) {
        let mut head = *self.head.index.get_mut();
        let mut tail = *self.tail.index.get_mut();
        let mut block = *self.head.block.get_mut();

        // Erase the lower bits.
        head &= !((1 << SHIFT) - 1);
        tail &= !((1 << SHIFT) - 1);

        unsafe {
            // Drop all values between `head` and `tail` and deallocate the heap-allocated blocks.
            while head != tail {
                let offset = ((head >> SHIFT) % LAP) as usize;

                if offset < BLOCK_CAP {
                    // Drop the value in the slot.
                    let slot = (*block).slots.get_unchecked(offset);
                    (*slot.value.get()).assume_init_drop();
                } else {
                    // Deallocate the block and move to the next one.
                    let next = *(*block).next.get_mut();
                    drop(Box::from_raw(block));
                    block = next;
                }

                head = head.wrapping_add(1 << SHIFT);
            }

            // Deallocate the last remaining block.
            if !block.is_null() {
                drop(Box::from_raw(block));
            }
        }
    }
}

unsafe impl<T: Send> Send for SegQueue<T> {}
unsafe impl<T: Send> Sync for SegQueue<T> {}

struct Position<T> {
    index: AtomicIndex,
    block: AtomicPtr<Block<T>>,
}

struct Block<T> {
    next: AtomicPtr<Block<T>>,
    slots: [Slot<T>; BLOCK_CAP],
}

impl<T> Block<T> {
    const LAYOUT: Layout = {
        let layout = Layout::new::<Self>();
        assert!(
            layout.size() != 0,
            "Block should never be zero-sized, as it has an AtomicPtr field"
        );
        layout
    };

    /// Creates an empty block.
    fn new() -> Box<Self> {
        let ptr = unsafe { alloc_zeroed(Self::LAYOUT) };
        if ptr.is_null() {
            handle_alloc_error(Self::LAYOUT)
        }
        unsafe { Box::from_raw(ptr.cast()) }
    }

    fn wait_next(&self) -> *mut Block<T> {
        let backoff = Backoff::new();
        loop {
            let next = self.next.load(Ordering::Acquire);
            if !next.is_null() {
                return next;
            }
            backoff.snooze();
        }
    }

    /// Sets the `DESTROY` bit in slots starting from `start` and destroys the block.
    unsafe fn destroy(this: *mut Block<T>, start: usize) {
        for i in start..BLOCK_CAP - 1 {
            let slot = unsafe { (*this).slots.get_unchecked(i) };

            if slot.state.load(Ordering::Acquire) & READ == 0
                && slot.state.fetch_or(DESTROY, Ordering::AcqRel) & READ == 0
            {
                return;
            }
        }
        drop(unsafe { Box::from_raw(this) });
    }

    /// Destroys the block. Only safe to call with exclusive access, when no other thread is using it.
    unsafe fn destroy_mut(this: *mut Self) {
        drop(unsafe { Box::from_raw(this) });
    }
}

struct Slot<T> {
    value: UnsafeCell<MaybeUninit<T>>,
    state: AtomicUsize,
}

impl<T> Slot<T> {
    fn wait_write(&self) {
        let backoff = Backoff::new();
        while self.state.load(Ordering::Acquire) & WRITE == 0 {
            backoff.snooze();
        }
    }
}

impl<T> IntoIterator for SegQueue<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { value: self }
    }
}

#[repr(transparent)]
pub struct IntoIter<T> {
    value: SegQueue<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let value = &mut self.value;
        let head = *value.head.index.get_mut();
        let tail = *value.tail.index.get_mut();
        if head >> SHIFT == tail >> SHIFT {
            None
        } else {
            let block = *value.head.block.get_mut();
            let offset = ((head >> SHIFT) % LAP) as usize;

            // SAFETY: We have mutable access to this, so we can read without
            // worrying about concurrency. Furthermore, we know this is
            // initialized because it is the value pointed at by `value.head`
            // and this is a non-empty queue.
            let item = unsafe {
                let slot = (*block).slots.get_unchecked(offset);
                slot.value.get().read().assume_init()
            };
            if offset + 1 == BLOCK_CAP {
                // Deallocate the block and move to the next one.
                // SAFETY: The block is initialized because we've been reading
                // from it this entire time. We can drop it b/c everything has
                // been read out of it, so nothing is pointing to it anymore.
                unsafe {
                    let next = *(*block).next.get_mut();
                    drop(Box::from_raw(block));
                    *value.head.block.get_mut() = next;
                }
                // The last value in a block is empty, so skip it
                *value.head.index.get_mut() = head.wrapping_add(2 << SHIFT);
                // Double-check that we're pointing to the first item in a block.
                debug_assert_eq!((*value.head.index.get_mut() >> SHIFT) % LAP, 0);
            } else {
                *value.head.index.get_mut() = head.wrapping_add(1 << SHIFT);
            }
            Some(item)
        }
    }
}
