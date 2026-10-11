use std::{
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::NonNull,
    sync::atomic::Ordering,
    task::{Context, Poll},
};

use super::{
    Header, PanicResult,
    state::{Snapshot, Strong},
};

#[repr(transparent)]
pub(super) struct Remote<'a> {
    ptr: NonNull<Header>,
    marker: PhantomData<&'a Header>,
}

impl<'a> Remote<'a> {
    pub fn new(ptr: NonNull<Header>) -> Self {
        Self {
            ptr,
            marker: PhantomData,
        }
    }

    pub fn schedule(&self) {
        let state = self.header().state.start_scheduling();

        if state.is_scheduled() || state.is_completed() || state.is_cancelled() {
            self.header().state.finish_scheduling();
            return;
        }

        let Some(shared) = (unsafe { self.header().shared.load(Ordering::Acquire).as_ref() })
        else {
            self.header().state.finish_scheduling();
            return;
        };

        super::panic_guard!();

        shared.pending.fetch_add(1, Ordering::Release);

        shared.sync.push(self.header().id);
        if let Some(ref waker) = shared.waker {
            waker.wake_by_ref();
        }

        self.header().state.finish_scheduling();
    }

    pub unsafe fn poll<T>(&self, cx: &mut Context<'_>) -> Poll<Option<PanicResult<T>>> {
        let mut state = self.state();

        loop {
            debug_assert!(state.has_result() || state.is_cancelled() || !state.is_completed());

            if state.has_result() {
                self.header().state.set_has_result::<Strong, false>();

                let mut res = MaybeUninit::<PanicResult<T>>::uninit();
                let target = NonNull::from_mut(&mut res).cast();
                unsafe { (self.header().vtable.take_result)(self.ptr, target) };

                break Poll::Ready(Some(unsafe { res.assume_init() }));
            } else if state.is_cancelled() {
                break Poll::Ready(None);
            }

            state = self.header().state.start_setting_waker();

            if state.has_result() {
                debug_assert!(state.is_completed());
                state = self.header().state.finish_setting_waker::<false>();

                continue;
            } else if state.is_cancelled() {
                self.header().state.finish_setting_waker::<false>();

                break Poll::Ready(None);
            } else if state.has_waker()
                && self
                    .header()
                    .waker
                    .with(|waker| cx.waker().will_wake(unsafe { (&*waker).assume_init_ref() }))
            {
                self.header().state.finish_setting_waker::<true>();
                break Poll::Pending;
            }

            self.header().waker.with_mut(|ptr| {
                super::panic_guard!();

                let waker = unsafe { &mut *ptr };

                if state.has_waker() {
                    unsafe { waker.assume_init_drop() };
                }

                waker.write(cx.waker().clone());
            });

            self.header().state.finish_setting_waker::<true>();

            break Poll::Pending;
        }
    }

    fn header(&self) -> &Header {
        unsafe { self.ptr.as_ref() }
    }

    fn state(&self) -> Snapshot {
        self.header().state.load::<Strong>()
    }
}
