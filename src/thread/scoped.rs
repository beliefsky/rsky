use std::{cell::Cell, marker, thread::LocalKey};

#[macro_export]
macro_rules! scoped_thread_local {
    ($(#[$attrs:meta])* $vis:vis static $name:ident: $ty:ty) => (
        $(#[$attrs])*
        $vis static $name: $crate::thread::ScopedKey<$ty> = unsafe {
            ::std::thread_local!(static FOO: ::std::cell::Cell<*const ()> = const {
                ::std::cell::Cell::new(::std::ptr::null())
            });

            $crate::thread::ScopedKey::new(&FOO)
        };
    )
}

pub struct ScopedKey<T> {
    inner: &'static LocalKey<Cell<*const ()>>,
    _marker: marker::PhantomData<T>,
}

impl<T> ScopedKey<T> {
    pub const unsafe fn new(inner: &'static LocalKey<Cell<*const ()>>) -> Self {
        Self {
            inner,
            _marker: marker::PhantomData,
        }
    }

    pub fn set<F, R>(&'static self, t: &T, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        struct Reset {
            key: &'static LocalKey<Cell<*const ()>>,
            val: *const (),
        }
        impl Drop for Reset {
            fn drop(&mut self) {
                self.key.with(|c| c.set(self.val));
            }
        }
        let prev = self.inner.with(|c| {
            let prev = c.get();
            c.set(t as *const T as *const ());
            prev
        });
        let _reset = Reset {
            key: self.inner,
            val: prev,
        };
        f()
    }

    pub fn with<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let val = self.inner.with(|c| c.get());
        assert!(
            !val.is_null(),
            "cannot access a scoped thread local variable without calling `set` first"
        );
        unsafe { f(&*(val as *const T)) }
    }

    pub fn is_set(&'static self) -> bool {
        self.inner.with(|c| !c.get().is_null())
    }
}

unsafe impl<T> Sync for ScopedKey<T> {}
