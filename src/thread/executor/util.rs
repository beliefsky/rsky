use std::task::Poll;

/// Create a guard that abort the process when the thread panicked before it's
/// out of scope.
///
/// If loom is enabled, this does nothing.
macro_rules! panic_guard {
    () => {
        let _b = {
            pub(crate) struct AbortOnPanic(());

            impl Drop for AbortOnPanic {
                fn drop(&mut self) {
                    if ::std::thread::panicking() {
                        ::std::process::abort()
                    }
                }
            }

            AbortOnPanic(())
        };
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
