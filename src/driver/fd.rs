use std::{
    cell::UnsafeCell,
    future::poll_fn,
    mem::ManuallyDrop,
    ops::Deref,
    os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd},
    panic::RefUnwindSafe,
    ptr,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::{Poll, Waker},
};

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
        // SAFETY: `this` is not dropped here.
        unsafe { ptr::read(&this.0) }
    }

    /// Try to take the inner owned fd.
    pub fn try_unwrap(self) -> Result<T, Self> {
        let inner = self.into_inner();
        Arc::try_unwrap(inner).map(|t| t.fd).map_err(|i| Self(i))
    }

    /// Wait and take the inner owned fd.
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

impl<T> RefUnwindSafe for Inner<T> {}

struct AtomicWaker {
    state: AtomicUsize,
    waker: UnsafeCell<Option<Waker>>,
}

impl AtomicWaker {
    const WAITING: usize = 0;
    const REGISTERING: usize = 0b01;

    const WAKING: usize = 0b10;

    pub const fn new() -> Self {
        Self {
            state: AtomicUsize::new(Self::WAITING),
            waker: UnsafeCell::new(None),
        }
    }

    pub fn register(&self, waker: &Waker) {
        match self
            .state
            .compare_exchange(
                Self::WAITING,
                Self::REGISTERING,
                Ordering::Acquire,
                Ordering::Acquire,
            )
            .unwrap_or_else(|x| x)
        {
            Self::WAITING => {
                unsafe {
                    // Locked acquired, update the waker cell

                    // Avoid cloning the waker if the old waker will awaken the same task.
                    match &*self.waker.get() {
                        Some(old_waker) if old_waker.will_wake(waker) => (),
                        _ => *self.waker.get() = Some(waker.clone()),
                    }

                    let res = self.state.compare_exchange(
                        Self::REGISTERING,
                        Self::WAITING,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    );

                    match res {
                        Ok(_) => {}
                        Err(actual) => {
                            debug_assert_eq!(actual, Self::REGISTERING | Self::WAKING);

                            let waker = (*self.waker.get()).take().unwrap();

                            self.state.swap(Self::WAITING, Ordering::AcqRel);

                            waker.wake();
                        }
                    }
                }
            }

            Self::WAKING => {
                waker.wake_by_ref();
            }
            state => {
                debug_assert!(
                    state == Self::REGISTERING || state == Self::REGISTERING | Self::WAKING
                );
            }
        }
    }

    pub fn wake(&self) {
        if let Some(waker) = self.take() {
            waker.wake();
        }
    }

    pub fn take(&self) -> Option<Waker> {
        match self.state.fetch_or(Self::WAKING, Ordering::AcqRel) {
            Self::WAITING => {
                let waker = unsafe { (*self.waker.get()).take() };

                let old_state = self.state.swap(Self::WAITING, Ordering::Release);
                debug_assert!(old_state == Self::WAKING);

                waker
            }
            state => {
                debug_assert!(
                    state == Self::REGISTERING
                        || state == Self::REGISTERING | Self::WAKING
                        || state == Self::WAKING
                );
                None
            }
        }
    }
}

impl Default for AtomicWaker {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for AtomicWaker {}
unsafe impl Sync for AtomicWaker {}
