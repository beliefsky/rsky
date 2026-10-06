use std::{
    cell::Cell,
    mem::{self, ManuallyDrop},
    pin::Pin,
    thread::ThreadId,
};

thread_local! {
    static THREAD_ID: Cell<ThreadId> = Cell::new(std::thread::current().id());
}

/// Get the current [`ThreadId`].
pub(crate) fn current_id() -> ThreadId {
    THREAD_ID.with(|id| id.get())
}

pub struct SendWrapper<T: ?Sized> {
    thread_id: ThreadId,
    data: ManuallyDrop<T>,
}

impl<T> SendWrapper<T> {
    #[inline]
    pub fn new(data: T) -> SendWrapper<T> {
        SendWrapper {
            data: ManuallyDrop::new(data),
            thread_id: current_id(),
        }
    }

    pub unsafe fn take_unchecked(self) -> T {
        // Prevent drop() from being called, as it would drop `self.data` twice
        let mut this = ManuallyDrop::new(self);

        unsafe { ManuallyDrop::take(&mut this.data) }
    }

    #[track_caller]
    pub fn take(self) -> T {
        if self.valid() {
            // SAFETY: the same thread as the creator
            unsafe { self.take_unchecked() }
        } else {
            invalid_deref()
        }
    }
}

impl<T: ?Sized> SendWrapper<T> {
    #[inline]
    pub fn valid(&self) -> bool {
        self.thread_id == current_id()
    }

    #[inline]
    pub unsafe fn get_unchecked(&self) -> &T {
        &self.data
    }

    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self) -> &mut T {
        &mut self.data
    }

    #[inline]
    pub unsafe fn get_unchecked_pinned(self: Pin<&Self>) -> Pin<&T> {
        unsafe { self.map_unchecked(|s| &*s.data) }
    }

    #[inline]
    pub unsafe fn get_unchecked_pinned_mut(self: Pin<&mut Self>) -> Pin<&mut T> {
        unsafe { self.map_unchecked_mut(|s| &mut *s.data) }
    }

    #[inline]
    pub fn get(&self) -> Option<&T> {
        if self.valid() { Some(&self.data) } else { None }
    }

    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.valid() {
            Some(&mut self.data)
        } else {
            None
        }
    }

    #[inline]
    pub fn get_pinned(self: Pin<&Self>) -> Option<Pin<&T>> {
        if self.valid() {
            Some(unsafe { self.get_unchecked_pinned() })
        } else {
            None
        }
    }

    #[inline]
    pub fn get_pinned_mut(self: Pin<&mut Self>) -> Option<Pin<&mut T>> {
        if self.valid() {
            Some(unsafe { self.get_unchecked_pinned_mut() })
        } else {
            None
        }
    }

    #[inline]
    pub fn tracker(&self) -> SendWrapper<()> {
        SendWrapper {
            data: ManuallyDrop::new(()),
            thread_id: self.thread_id,
        }
    }
}

unsafe impl<T: ?Sized> Send for SendWrapper<T> {}
unsafe impl<T: ?Sized> Sync for SendWrapper<T> {}

impl<T: ?Sized> Drop for SendWrapper<T> {
    #[track_caller]
    fn drop(&mut self) {
        if !mem::needs_drop::<T>() || self.valid() {
            unsafe {
                ManuallyDrop::drop(&mut self.data);
            }
        } else {
            invalid_drop()
        }
    }
}

impl<T: Clone> Clone for SendWrapper<T> {
    #[track_caller]
    fn clone(&self) -> Self {
        Self::new(self.get().unwrap_or_else(|| invalid_deref()).clone())
    }
}

#[cold]
#[inline(never)]
#[track_caller]
fn invalid_deref() -> ! {
    const DEREF_ERROR: &str = "Accessed SendWrapper<T> variable from a thread different to the \
                               one it has been created with.";

    panic!("{}", DEREF_ERROR)
}

#[cold]
#[inline(never)]
#[track_caller]
fn invalid_drop() {
    const DROP_ERROR: &str = "Dropped SendWrapper<T> variable from a thread different to the one \
                              it has been created with.";

    if !std::thread::panicking() {
        // panic because of dropping from wrong thread
        // only do this while not unwinding (could be caused by deref from wrong thread)
        panic!("{}", DROP_ERROR)
    }
}
