use std::{
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::NonNull,
    sync::atomic::Ordering::*,
    task::{Context, Poll},
};

use super::{
    Header, PanicResult,
    state::{Snapshot, Strong, Weak},
};

#[repr(transparent)]
pub(super) struct Local<'a> {
    ptr: NonNull<Header>,
    marker: PhantomData<&'a Header>,
}

impl<'a> Local<'a> {
    #[inline(always)]
    pub unsafe fn new(ptr: NonNull<Header>) -> Self {
        Self {
            ptr,
            marker: PhantomData,
        }
    }

    pub fn schedule(&self) {
        let Some(shared) = (unsafe { self.header().shared.load(Relaxed).as_ref() }) else {
            return;
        };

        let queue = unsafe { shared.queue.get_unchecked() };

        shared.drain_sync(queue);

        queue.make_hot(self.header().id);

        if let Some(ref waker) = shared.waker {
            waker.wake_by_ref()
        }
    }

    pub unsafe fn poll<T>(&self, cx: &mut Context<'_>) -> Poll<Option<PanicResult<T>>> {
        let state = self.state();

        debug_assert!(state.has_result() || !state.is_completed() || state.is_cancelled());

        if state.has_result() {
            self.header().state.set_has_result::<Strong, false>();

            let mut res = MaybeUninit::<PanicResult<T>>::uninit();
            let target = NonNull::from_mut(&mut res).cast();
            unsafe { (self.header().vtable.take_result)(self.ptr, target) };

            return Poll::Ready(Some(unsafe { res.assume_init() }));
        }

        if state.is_cancelled() {
            return Poll::Ready(None);
        }

        if !state.is_completed() {
            return self.header().waker.with_mut(|waker| {
                super::panic_guard!();
                let waker = unsafe { &mut *waker };
                if state.has_waker() {
                    if cx.waker().will_wake(unsafe { waker.assume_init_ref() }) {
                        return Poll::Pending;
                    }
                    unsafe { waker.assume_init_drop() };
                }
                waker.write(cx.waker().clone());
                self.header().state.set_has_waker::<Weak, true>();

                Poll::Pending
            });
        }

        unreachable!("Task is completed but has no result")
    }

    fn header(&self) -> &Header {
        unsafe { self.ptr.as_ref() }
    }

    fn state(&self) -> Snapshot {
        self.header().state.load::<Weak>()
    }
}
