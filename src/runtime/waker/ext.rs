use std::{
    mem::ManuallyDrop,
    ptr,
    sync::Arc,
    task::{RawWaker, RawWakerVTable, Waker},
};

use crate::{runtime::Ext, thread::SendWrapper};

// `static` guarantees the uniqueness of vtable in memory
static EXT_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    ExtWaker::clone,
    ExtWaker::wake,
    ExtWaker::wake_by_ref,
    ExtWaker::drop,
);

static OWNED_EXT_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    OwnedExtWaker::clone,
    OwnedExtWaker::wake,
    OwnedExtWaker::wake_by_ref,
    OwnedExtWaker::drop,
);

#[derive(Clone)]
pub(crate) struct ExtWaker<'a> {
    waker: &'a Waker,
    // `SendWrapper<&Ext>` will not panic when being dropped on other thread since references
    // doesn't need drop
    ext: SendWrapper<&'a Ext>,
}

impl<'a> ExtWaker<'a> {
    unsafe fn from_raw<'s>(ptr: *const ()) -> &'s Self {
        unsafe { &*ptr.cast::<Self>() }
    }

    unsafe fn clone(ptr: *const ()) -> RawWaker {
        let this = unsafe { Self::from_raw(ptr) };

        if let Some(owned) = this.to_owned() {
            let waker = ManuallyDrop::new(owned.into_std());
            RawWaker::new(waker.data(), waker.vtable())
        } else {
            let waker = ManuallyDrop::new(this.waker.clone());
            RawWaker::new(waker.data(), waker.vtable())
        }
    }

    unsafe fn wake(_: *const ()) {
        unreachable!("ExtWaker will only be accessed with reference")
    }

    unsafe fn wake_by_ref(ptr: *const ()) {
        unsafe { Self::from_raw(ptr) }.waker.wake_by_ref();
    }

    unsafe fn drop(_: *const ()) {
        // `ExtWaker` only contains reference, no need to drop.
    }

    fn to_owned(&self) -> Option<OwnedExtWaker> {
        let ext_data = self.ext.get().copied()?.to_owned();
        let ext = ManuallyDrop::new(SendWrapper::new(ext_data));

        Some(OwnedExtWaker(Arc::new(Inner {
            waker: self.waker.clone(),
            ext,
        })))
    }
}

struct OwnedExtWaker(Arc<Inner>);

struct Inner {
    waker: Waker,
    ext: ManuallyDrop<SendWrapper<Ext>>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        if self.ext.valid() {
            unsafe { ManuallyDrop::drop(&mut self.ext) };
        }
    }
}

impl OwnedExtWaker {
    unsafe fn clone(ptr: *const ()) -> RawWaker {
        unsafe { Arc::increment_strong_count(ptr.cast::<Inner>()) };
        RawWaker::new(ptr, &OWNED_EXT_WAKER_VTABLE)
    }

    unsafe fn wake(ptr: *const ()) {
        unsafe { Arc::from_raw(ptr.cast::<Inner>()) }
            .waker
            .wake_by_ref();
    }

    unsafe fn wake_by_ref(ptr: *const ()) {
        unsafe { Self::from_raw(ptr) }.waker.wake_by_ref();
    }

    unsafe fn drop(ptr: *const ()) {
        _ = unsafe { Arc::from_raw(ptr.cast::<Inner>()) };
    }

    unsafe fn from_raw<'b>(ptr: *const ()) -> &'b Inner {
        unsafe { &*ptr.cast::<Inner>() }
    }

    fn into_std(self) -> Waker {
        unsafe {
            Waker::from_raw(RawWaker::new(
                Arc::into_raw(self.0).cast::<()>(),
                &OWNED_EXT_WAKER_VTABLE,
            ))
        }
    }
}

pub(crate) fn get_waker(waker: &Waker) -> &Waker {
    if ptr::eq(waker.vtable(), &EXT_WAKER_VTABLE) {
        get_waker(unsafe { ExtWaker::from_raw(waker.data()) }.waker)
    } else if ptr::eq(waker.vtable(), &OWNED_EXT_WAKER_VTABLE) {
        get_waker(&unsafe { OwnedExtWaker::from_raw(waker.data()) }.waker)
    } else {
        waker
    }
}

pub(crate) fn get_ext(waker: &Waker) -> Option<&Ext> {
    if ptr::eq(waker.vtable(), &EXT_WAKER_VTABLE) {
        unsafe { ExtWaker::from_raw(waker.data()) }
            .ext
            .get()
            .copied()
    } else if ptr::eq(waker.vtable(), &OWNED_EXT_WAKER_VTABLE) {
        unsafe { OwnedExtWaker::from_raw(waker.data()) }.ext.get()
    } else {
        None
    }
}
