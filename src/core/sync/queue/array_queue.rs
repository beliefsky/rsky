use crate::core::sync::{Backoff, CachePadded};
use alloc::boxed::Box;
use core::{
    cell::UnsafeCell,
    mem,
    mem::MaybeUninit,
    sync::atomic::{self, Ordering},
};

#[cfg(target_has_atomic = "64")]
type AtomicIndex = atomic::AtomicU64;
#[cfg(target_has_atomic = "64")]
type Index = u64;
#[cfg(not(target_has_atomic = "64"))]
type AtomicIndex = atomic::AtomicUsize;
#[cfg(not(target_has_atomic = "64"))]
type Index = usize;

// A bounded multi-producer multi-consumer queue.
///
/// This queue allocates a fixed-capacity buffer on construction, which is used to store pushed
/// elements. The queue cannot hold more elements than the buffer allows. Attempting to push an
/// element into a full queue will fail. Alternatively, [`force_push`] makes it possible for
/// this queue to be used as a ring-buffer. Having a buffer allocated upfront makes this queue
/// a bit faster than [`SegQueue`].
pub struct ArrayQueue<T> {
    head: CachePadded<AtomicIndex>,
    tail: CachePadded<AtomicIndex>,
    buffer: Box<[Slot<T>]>,
    one_lap: Index,
}

impl<T> ArrayQueue<T> {
    // Creates a new bounded queue with the given capacity.
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0, "capacity must be non-zero");

        // Head is initialized to `{ lap: 0, index: 0 }`.
        // Tail is initialized to `{ lap: 0, index: 0 }`.
        let head = 0;
        let tail = 0;

        // Allocate a buffer of `cap` slots initialized
        // with stamps.
        let buffer: Box<[Slot<T>]> = (0..cap)
            .map(|i| {
                // Set the stamp to `{ lap: 0, index: i }`.
                Slot {
                    stamp: AtomicIndex::new(i as Index),
                    value: UnsafeCell::new(MaybeUninit::uninit()),
                }
            })
            .collect();

        // One lap is the smallest power of two greater than `cap`.
        let one_lap = (cap as Index)
            .checked_add(1)
            .and_then(Index::checked_next_power_of_two)
            .expect("queue capacity is too large");

        Self {
            buffer,
            one_lap,
            head: CachePadded::new(AtomicIndex::new(head)),
            tail: CachePadded::new(AtomicIndex::new(tail)),
        }
    }

    /// Returns the capacity of the queue.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Returns `true` if the queue is empty.
    pub fn is_empty(&self) -> bool {
        let head = self.head.load(Ordering::SeqCst);
        let tail = self.tail.load(Ordering::SeqCst);

        tail == head
    }

    /// Returns `true` if the queue is full.
    pub fn is_full(&self) -> bool {
        let tail = self.tail.load(Ordering::SeqCst);
        let head = self.head.load(Ordering::SeqCst);

        // Is the head lagging one lap behind tail?
        //
        // Note: If the tail changes just before we load the head, that means there was a moment
        // when the queue was not full, so it is safe to just return `false`.
        head.wrapping_add(self.one_lap) == tail
    }

    /// Attempts to push an element into the queue.
    ///
    /// If the queue is full, the element is returned back as an error.
    pub fn push(&self, value: T) -> Result<(), T> {
        self.push_or_else(value, |v, tail, _, _| {
            let head = self.head.load(Ordering::Relaxed);

            // If the head lags one lap behind the tail as well...
            if head.wrapping_add(self.one_lap) == tail {
                // ...then the queue is full.
                Err(v)
            } else {
                Ok(v)
            }
        })
    }

    /// Returns the number of elements in the queue.
    pub fn len(&self) -> usize {
        loop {
            // Load the tail, then load the head.
            let tail = self.tail.load(Ordering::SeqCst);
            let head = self.head.load(Ordering::SeqCst);

            // If the tail didn't change, we've got consistent values to work with.
            if self.tail.load(Ordering::SeqCst) == tail {
                let hix = (head & (self.one_lap - 1)) as usize;
                let tix = (tail & (self.one_lap - 1)) as usize;

                return if hix < tix {
                    tix - hix
                } else if hix > tix {
                    self.capacity() - hix + tix
                } else if tail == head {
                    0
                } else {
                    self.capacity()
                };
            }
        }
    }

    /// Attempts to push an element using an exclusive reference of the queue.
    ///
    /// Atomic operations and checks are omitted
    pub fn push_mut(&mut self, value: T) -> Result<(), T> {
        let tail = *self.tail.get_mut();
        let head = *self.head.get_mut();

        if head.wrapping_add(self.one_lap) == tail {
            return Err(value);
        }

        let index = (tail & (self.one_lap - 1)) as usize;
        let lap = tail & !(self.one_lap - 1);
        let new_tail = if index + 1 < self.capacity() {
            tail + 1
        } else {
            lap.wrapping_add(self.one_lap)
        };

        *self.tail.get_mut() = new_tail;

        let slot = unsafe { self.buffer.get_unchecked_mut(index) };
        unsafe {
            slot.value.get().write(MaybeUninit::new(value));
        }
        *slot.stamp.get_mut() = tail + 1;

        Ok(())
    }

    /// Pushes an element into the queue, replacing the oldest element if necessary.
    ///
    /// If the queue is full, the oldest element is replaced and returned,
    /// otherwise `None` is returned.
    pub fn force_push(&self, value: T) -> Option<T> {
        self.push_or_else(value, |v, tail, new_tail, slot| {
            let head = tail.wrapping_sub(self.one_lap);
            let new_head = new_tail.wrapping_sub(self.one_lap);

            // Try moving the head.
            if self
                .head
                .compare_exchange_weak(head, new_head, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                // Move the tail.
                self.tail.store(new_tail, Ordering::SeqCst);

                // Swap the previous value.
                let old = unsafe { slot.value.get().replace(MaybeUninit::new(v)).assume_init() };

                // Update the stamp.
                slot.stamp.store(tail + 1, Ordering::Release);

                Err(old)
            } else {
                Ok(v)
            }
        })
        .err()
    }

    fn push_or_else<F>(&self, mut value: T, f: F) -> Result<(), T>
    where
        F: Fn(T, Index, Index, &Slot<T>) -> Result<T, T>,
    {
        let backoff = Backoff::new();
        let mut tail = self.tail.load(Ordering::Relaxed);

        loop {
            // Deconstruct the tail.
            let index = (tail & (self.one_lap - 1)) as usize;
            let lap = tail & !(self.one_lap - 1);

            let new_tail = if index + 1 < self.capacity() {
                // Same lap, incremented index.
                // Set to `{ lap: lap, index: index + 1 }`.
                tail + 1
            } else {
                // One lap forward, index wraps around to zero.
                // Set to `{ lap: lap.wrapping_add(1), index: 0 }`.
                lap.wrapping_add(self.one_lap)
            };

            // Inspect the corresponding slot.
            debug_assert!(index < self.buffer.len());
            let slot = unsafe { self.buffer.get_unchecked(index) };
            let stamp = slot.stamp.load(Ordering::Acquire);

            // If the tail and the stamp match, we may attempt to push.
            if tail == stamp {
                // Try moving the tail.
                match self.tail.compare_exchange_weak(
                    tail,
                    new_tail,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // Write the value into the slot and update the stamp.
                        unsafe {
                            slot.value.get().write(MaybeUninit::new(value));
                        }
                        slot.stamp.store(tail + 1, Ordering::Release);
                        return Ok(());
                    }
                    Err(t) => {
                        tail = t;
                        backoff.spin();
                    }
                }
            } else if stamp.wrapping_add(self.one_lap) == tail + 1 {
                atomic::fence(Ordering::SeqCst);
                value = f(value, tail, new_tail, slot)?;
                backoff.spin();
                tail = self.tail.load(Ordering::Relaxed);
            } else {
                // Snooze because we need to wait for the stamp to get updated.
                backoff.snooze();
                tail = self.tail.load(Ordering::Relaxed);
            }
        }
    }

    // Attempts to pop an element from the queue.
    ///
    /// If the queue is empty, `None` is returned.
    pub fn pop(&self) -> Option<T> {
        let backoff = Backoff::new();
        let mut head = self.head.load(Ordering::Relaxed);

        loop {
            // Deconstruct the head.
            let index = (head & (self.one_lap - 1)) as usize;
            let lap = head & !(self.one_lap - 1);

            // Inspect the corresponding slot.
            debug_assert!(index < self.buffer.len());
            let slot = unsafe { self.buffer.get_unchecked(index) };
            let stamp = slot.stamp.load(Ordering::Acquire);

            // If the stamp is ahead of the head by 1, we may attempt to pop.
            if head + 1 == stamp {
                let new = if index + 1 < self.capacity() {
                    // Same lap, incremented index.
                    // Set to `{ lap: lap, index: index + 1 }`.
                    head + 1
                } else {
                    // One lap forward, index wraps around to zero.
                    // Set to `{ lap: lap.wrapping_add(1), index: 0 }`.
                    lap.wrapping_add(self.one_lap)
                };

                // Try moving the head.
                match self.head.compare_exchange_weak(
                    head,
                    new,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        // Read the value from the slot and update the stamp.
                        let msg = unsafe { slot.value.get().read().assume_init() };
                        slot.stamp
                            .store(head.wrapping_add(self.one_lap), Ordering::Release);
                        return Some(msg);
                    }
                    Err(h) => {
                        head = h;
                        backoff.spin();
                    }
                }
            } else if stamp == head {
                atomic::fence(Ordering::SeqCst);
                let tail = self.tail.load(Ordering::Relaxed);

                // If the tail equals the head, that means the channel is empty.
                if tail == head {
                    return None;
                }

                backoff.spin();
                head = self.head.load(Ordering::Relaxed);
            } else {
                // Snooze because we need to wait for the stamp to get updated.
                backoff.snooze();
                head = self.head.load(Ordering::Relaxed);
            }
        }
    }

    /// Attempts to pop an element using an exclusive reference of the queue.
    ///
    /// Due to having an exclusive reference, atomic operations and checks are omitted
    pub fn pop_mut(&mut self) -> Option<T> {
        let head = *self.head.get_mut();
        let tail = *self.tail.get_mut();

        // If the tail equals the head, that means the channel is empty.
        if tail == head {
            return None;
        }
        let index = (head & (self.one_lap - 1)) as usize;
        let lap = head & !(self.one_lap - 1);

        // Inspect the corresponding slot.
        debug_assert!(index < self.buffer.len());

        let new = if index + 1 < self.capacity() {
            // Same lap, incremented index.
            // Set to `{ lap: lap, index: index + 1 }`.
            head + 1
        } else {
            // One lap forward, index wraps around to zero.
            // Set to `{ lap: lap.wrapping_add(1), index: 0 }`.
            lap.wrapping_add(self.one_lap)
        };

        let slot = unsafe { self.buffer.get_unchecked_mut(index) };

        let msg = unsafe { slot.value.get().read().assume_init() };
        *slot.stamp.get_mut() = head.wrapping_add(self.one_lap);
        *self.head.get_mut() = new;
        Some(msg)
    }
}

impl<T> Drop for ArrayQueue<T> {
    fn drop(&mut self) {
        if mem::needs_drop::<T>() {
            // Get the index of the head.
            let head = *self.head.get_mut();
            let tail = *self.tail.get_mut();

            let hix = (head & (self.one_lap - 1)) as usize;
            let tix = (tail & (self.one_lap - 1)) as usize;

            let len = if hix < tix {
                tix - hix
            } else if hix > tix {
                self.capacity() - hix + tix
            } else if tail == head {
                0
            } else {
                self.capacity()
            };

            // Loop over all slots that hold a message and drop them.
            for i in 0..len {
                // Compute the index of the next slot holding a message.
                let index = if hix + i < self.capacity() {
                    hix + i
                } else {
                    hix + i - self.capacity()
                };

                unsafe {
                    debug_assert!(index < self.buffer.len());
                    let slot = self.buffer.get_unchecked_mut(index);
                    (*slot.value.get()).assume_init_drop();
                }
            }
        }
    }
}

unsafe impl<T: Send> Sync for ArrayQueue<T> {}
unsafe impl<T: Send> Send for ArrayQueue<T> {}

impl<T> IntoIterator for ArrayQueue<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { value: self }
    }
}

struct Slot<T> {
    stamp: AtomicIndex,
    value: UnsafeCell<MaybeUninit<T>>,
}

#[repr(transparent)]
pub struct IntoIter<T> {
    value: ArrayQueue<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let value = &mut self.value;
        let head = *value.head.get_mut();
        if value.head.get_mut() != value.tail.get_mut() {
            let index = (head & (value.one_lap - 1)) as usize;
            let lap = head & !(value.one_lap - 1);
            // SAFETY: We have mutable access to this, so we can read without
            // worrying about concurrency. Furthermore, we know this is
            // initialized because it is the value pointed at by `value.head`
            // and this is a non-empty queue.
            let val = unsafe {
                debug_assert!(index < value.buffer.len());
                let slot = value.buffer.get_unchecked_mut(index);
                slot.value.get().read().assume_init()
            };
            let new = if index + 1 < value.capacity() {
                // Same lap, incremented index.
                // Set to `{ lap: lap, index: index + 1 }`.
                head + 1
            } else {
                // One lap forward, index wraps around to zero.
                // Set to `{ lap: lap.wrapping_add(1), index: 0 }`.
                lap.wrapping_add(value.one_lap)
            };
            *value.head.get_mut() = new;
            Some(val)
        } else {
            None
        }
    }
}
