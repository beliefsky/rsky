use std::{
    hash::Hash,
    io,
    ops::{Deref, DerefMut},
    task::Waker,
};

use thin_cell::unsync::{Inner, Ref, ThinCell, Weak};

use crate::{
    buf::{BufResult, IntoInner},
    driver::{Carry, Extra, OpCode, OpEntry, PushEntry, control::Carrier},
};

#[repr(transparent)]
pub struct Key<T> {
    erased: ErasedKey,
    _p: std::marker::PhantomData<T>,
}

impl<T> Key<T> {
    // pub(crate) fn into_raw(self) -> usize {
    //     self.erased.into_raw()
    // }

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
    /// Create [`RawOp`] and get the [`Key`] to it.
    pub(crate) fn new(op: T, extra: impl Into<Extra>) -> Self {
        let erased = ErasedKey::new(op, extra.into());

        Self {
            erased,
            _p: std::marker::PhantomData,
        }
    }
}

impl<T> Clone for Key<T> {
    fn clone(&self) -> Self {
        Self {
            erased: self.erased.clone(),
            _p: std::marker::PhantomData,
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
    inner: ThinCell<RawOp<dyn Carry>>,
}

impl ErasedKey {
    pub(crate) fn new<T: OpCode + 'static>(op: T, extra: Extra) -> Self {
        let raw_op = RawOp {
            extra,
            cancelled: false,
            result: PushEntry::Pending(None),
            carrier: Carrier::new(op),
        };
        let mut inner = ThinCell::new(raw_op);
        unsafe { inner.borrow_unchecked().carrier.init() };
        Self {
            inner: unsafe { inner.unsize(|p| p as *const Inner<RawOp<dyn Carry>>) },
        }
    }

    pub(crate) unsafe fn from_raw(user_data: usize) -> Self {
        let inner = unsafe { ThinCell::from_raw(user_data as *mut ()) };
        Self { inner }
    }

    pub(crate) fn as_raw(&self) -> usize {
        self.inner.as_ptr() as _
    }

    pub(crate) fn into_raw(self) -> usize {
        self.inner.leak() as _
    }

    #[inline]
    pub(crate) fn borrow(&self) -> Ref<'_, RawOp<dyn Carry>> {
        self.inner.borrow()
    }

    pub(crate) fn has_result(&self) -> bool {
        self.borrow().result.is_ready()
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

    pub(crate) fn set_result(&self, res: io::Result<usize>) {
        let mut this = self.borrow();
        {
            let RawOp { extra, carrier, .. } = &mut *this;
            unsafe { Carry::set_result(carrier, &res, extra) };
        }
        if let PushEntry::Pending(Some(w)) =
            std::mem::replace(&mut this.result, PushEntry::Ready(res))
        {
            w.wake();
        }
    }

    unsafe fn take_result<T: OpCode>(self) -> BufResult<usize, T> {
        // SAFETY: Caller guarantees that `T` is the actual concrete type.
        let this = unsafe { self.inner.downcast_unchecked::<RawOp<Carrier<T>>>() };
        let op = this.try_unwrap().map_err(|_| ()).expect("Key not unique");
        let res = op.result.take_ready().expect("Result not ready");
        BufResult(res, op.carrier.into_inner())
    }
}

impl PartialEq for ErasedKey {
    fn eq(&self, other: &Self) -> bool {
        self.inner.ptr_eq(&other.inner)
    }
}

impl Eq for ErasedKey {}

impl Hash for ErasedKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.inner.as_ptr() as usize).hash(state)
    }
}

impl Unpin for ErasedKey {}

#[repr(C)]
pub(crate) struct RawOp<M: ?Sized> {
    extra: Extra,
    cancelled: bool,
    result: PushEntry<Option<Waker>, io::Result<usize>>,
    pub(crate) carrier: M,
}

impl<C: ?Sized> RawOp<C> {
    pub fn extra(&self) -> &Extra {
        &self.extra
    }

    pub fn extra_mut(&mut self) -> &mut Extra {
        &mut self.extra
    }
}

impl<C: Carry + ?Sized> RawOp<C> {
    pub fn create_entry(&mut self) -> OpEntry {
        self.carrier.create_entry().with_extra(&self.extra)
    }
}
