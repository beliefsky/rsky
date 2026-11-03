use crate::core::sync::task::AtomicWaker;
use alloc::sync::Arc;
use core::{
    future::poll_fn,
    mem::ManuallyDrop,
    ops::Deref,
    ptr,
    sync::atomic::{AtomicBool, Ordering},
    task::Poll,
};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};

#[repr(transparent)]
pub struct SharedFd<T>(Arc<Inner<T>>);

impl<T: AsFd> SharedFd<T> {
    pub fn new(fd: T) -> Self {
        unsafe { Self::new_unchecked(fd) }
    }
}

impl<T> SharedFd<T> {
    pub unsafe fn new_unchecked(fd: T) -> Self {
        Self(Arc::new(Inner {
            fd,
            waits: AtomicBool::new(false),
            waker: AtomicWaker::new(),
        }))
    }

    fn into_inner(self) -> Arc<Inner<T>> {
        let this = ManuallyDrop::new(self);

        unsafe { ptr::read(&this.0) }
    }

    pub fn try_unwrap(self) -> Result<T, Self> {
        let inner = self.into_inner();
        Arc::try_unwrap(inner).map(|t| t.fd).map_err(|i| Self(i))
    }

    pub fn take(self) -> impl Future<Output = Option<T>> {
        let inner = self.into_inner();

        async move {
            if !inner.waits.swap(true, Ordering::AcqRel) {
                let mut inner = Some(inner);
                poll_fn(move |cx| {
                    let i = inner.take().unwrap();
                    let this = match Arc::try_unwrap(i) {
                        Ok(fd) => return Poll::Ready(Some(fd.fd)),
                        Err(this) => this,
                    };

                    this.waker.register(cx.waker());

                    match Arc::try_unwrap(this) {
                        Ok(fd) => Poll::Ready(Some(fd.fd)),
                        Err(tt) => {
                            inner = Some(tt);
                            Poll::Pending
                        }
                    }
                })
                .await
            } else {
                None
            }
        }
    }
}

impl<T> Clone for SharedFd<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Drop for SharedFd<T> {
    fn drop(&mut self) {
        if Arc::strong_count(&self.0) == 2 && self.0.waits.load(Ordering::Acquire) {
            self.0.waker.wake()
        }
    }
}

impl<T: AsFd> AsFd for SharedFd<T> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.fd.as_fd()
    }
}

impl<T: AsFd> AsRawFd for SharedFd<T> {
    fn as_raw_fd(&self) -> RawFd {
        self.as_fd().as_raw_fd()
    }
}
impl<T> Deref for SharedFd<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.fd
    }
}

struct Inner<T> {
    fd: T,
    waits: AtomicBool,
    waker: AtomicWaker,
}
