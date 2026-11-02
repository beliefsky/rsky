use core::{
    hash,
    hash::Hash,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    ptr,
    task::Waker,
};
use std::io;

use thin_cell::unsync::{Inner, Ref, ThinCell, Weak};

use crate::{
    driver::{Extra, OpCode, OpEntry, PushEntry},
    io::BufResult,
};

#[repr(transparent)]
pub struct Key<T> {
    erased: ErasedKey,
    _p: PhantomData<T>,
}

impl<T> Key<T> {
    pub(crate) fn erase(self) -> ErasedKey {
        self.erased
    }
}

impl<T: OpCode> Key<T> {
    pub(crate) fn take_result(self) -> BufResult<usize, T> {
        unsafe { self.erased.take_result::<T>() }
    }
}

impl<T: OpCode + 'static> Key<T> {
    pub(crate) fn new(op: T, extra: impl Into<Extra>) -> Self {
        let erased = ErasedKey::new(op, extra.into());

        Self {
            erased,
            _p: PhantomData,
        }
    }
}

impl<T> Clone for Key<T> {
    fn clone(&self) -> Self {
        Self {
            erased: self.erased.clone(),
            _p: PhantomData,
        }
    }
}

impl<T> Deref for Key<T> {
    type Target = ErasedKey;

    fn deref(&self) -> &Self::Target {
        &self.erased
    }
}

impl<T> DerefMut for Key<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.erased
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct ErasedKey {
    inner: ThinCell<RawOp<dyn OpCode>>,
}

impl ErasedKey {
    pub(crate) fn new<T: OpCode + 'static>(op: T, extra: Extra) -> Self {
        let raw_op = RawOp {
            extra,
            cancelled: false,
            result: PushEntry::Pending(None),
            op,
        };
        let inner = ThinCell::new(raw_op);
        Self {
            inner: unsafe { inner.unsize(|p| p as *const Inner<RawOp<dyn OpCode>>) },
        }
    }

    pub(crate) unsafe fn from_raw(user_data: usize) -> Self {
        let inner = unsafe { ThinCell::from_raw(user_data as *mut ()) };
        Self { inner }
    }

    pub(crate) fn downgrade(&self) -> WeakKey {
        WeakKey {
            inner: self.inner.downgrade(),
        }
    }

    pub(crate) fn as_raw(&self) -> usize {
        self.inner.as_ptr() as _
    }

    pub(crate) fn into_raw(self) -> usize {
        self.inner.leak() as _
    }

    #[inline]
    pub(crate) fn borrow(&self) -> Ref<'_, RawOp<dyn OpCode>> {
        self.inner.borrow()
    }

    pub(crate) fn set_cancelled(&self) -> bool {
        let mut op = self.borrow();
        mem::replace(&mut op.cancelled, true)
    }

    pub(crate) fn is_unique(&self) -> bool {
        ThinCell::count(&self.inner) == 1
    }

    pub(crate) fn has_result(&self) -> bool {
        self.borrow().result.is_ready()
    }

    pub(crate) fn set_result(&self, res: io::Result<usize>) {
        let mut this = self.borrow();
        {
            let RawOp { extra, op, .. } = &mut *this;
            unsafe { op.set_result(&res, extra) };
        }
        if let PushEntry::Pending(Some(w)) = mem::replace(&mut this.result, PushEntry::Ready(res)) {
            w.wake();
        }
    }

    unsafe fn take_result<T: OpCode>(self) -> BufResult<usize, T> {
        let this = unsafe { self.inner.downcast_unchecked::<RawOp<T>>() };
        let raw_op = this.try_unwrap().map_err(|_| ()).expect("Key not unique");
        let res = raw_op.result.take_ready().expect("Result not ready");
        BufResult::new(res, raw_op.op)
    }

    pub(crate) fn swap_extra(&self, extra: Extra) -> Extra {
        mem::replace(&mut self.borrow().extra, extra)
    }

    pub(crate) fn set_waker(&self, waker: &Waker) {
        let PushEntry::Pending(w) = &mut self.borrow().result else {
            return;
        };

        if w.as_ref().is_some_and(|w| w.will_wake(waker)) {
            return;
        }

        *w = Some(waker.clone());
    }
}

impl PartialEq for ErasedKey {
    fn eq(&self, other: &Self) -> bool {
        self.inner.ptr_eq(&other.inner)
    }
}

impl Eq for ErasedKey {}

impl Hash for ErasedKey {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        (self.inner.as_ptr() as usize).hash(state)
    }
}

impl Unpin for ErasedKey {}

#[repr(C)]
pub(crate) struct RawOp<P: OpCode + ?Sized> {
    extra: Extra,
    cancelled: bool,
    result: PushEntry<Option<Waker>, io::Result<usize>>,
    pub(crate) op: P,
}

impl<P: OpCode + ?Sized> RawOp<P> {
    pub fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }
    pub fn create_entry(&mut self) -> OpEntry {
        self.op.create_entry().with_extra(&self.extra)
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub(crate) struct WeakKey {
    inner: Weak<RawOp<dyn OpCode>>,
}

impl WeakKey {
    pub(crate) fn upgrade(&self) -> Option<ErasedKey> {
        Some(ErasedKey {
            inner: self.inner.upgrade()?,
        })
    }

    pub(crate) fn as_ptr(&self) -> *const () {
        self.inner.as_ptr()
    }
}

impl PartialEq for WeakKey {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.inner.as_ptr(), other.inner.as_ptr())
    }
}

impl Eq for WeakKey {}

impl Hash for WeakKey {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        (self.inner.as_ptr() as usize).hash(state)
    }
}
