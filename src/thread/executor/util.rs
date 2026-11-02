use core::task::Poll;

pub(crate) struct AbortOnPanic;

impl AbortOnPanic {
    pub(crate) fn unless_unwinding() -> Option<Self> {
        if std::thread::panicking() {
            None
        } else {
            Some(Self)
        }
    }
}

impl Drop for AbortOnPanic {
    fn drop(&mut self) {
        if std::thread::panicking() {
            std::process::abort()
        }
    }
}

macro_rules! panic_guard {
    () => {
        let _b = crate::thread::executor::util::AbortOnPanic;
    };
    (skip_if_unwinding) => {
        let _b = crate::thread::executor::util::AbortOnPanic::unless_unwinding();
    };
}

pub(crate) use panic_guard;

#[inline(always)]
pub(crate) fn transpose<T, E>(poll: Result<Poll<T>, E>) -> Poll<Result<T, E>> {
    match poll {
        Ok(Poll::Pending) => Poll::Pending,
        Ok(Poll::Ready(t)) => Poll::Ready(Ok(t)),
        Err(e) => Poll::Ready(Err(e)),
    }
}

macro_rules! assert_not_impl {
    ($x:ty, $($t:path),+ $(,)*) => {
        const _: fn() -> () = || {
            struct Check<T: ?Sized>(T);
            trait AmbiguousIfImpl<A> { fn some_item() { } }

            impl<T: ?Sized> AmbiguousIfImpl<()> for Check<T> { }
            impl<T: ?Sized $(+ $t)*> AmbiguousIfImpl<u8> for Check<T> { }

            <Check::<$x> as AmbiguousIfImpl<_>>::some_item()
        };
    };
}

pub(crate) use assert_not_impl;
