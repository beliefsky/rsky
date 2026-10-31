use super::super::sys;
use core::mem::MaybeUninit;

pub const IOVEC_MAX_LEN: usize = sys::IOVEC_MAX_LEN;

pub struct IoVec {
    inner: sys::IoVec,
}

impl IoVec {
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub unsafe fn from_ref(s: &[u8]) -> IoVec {
        IoVec {
            inner: unsafe { sys::IoVec::from_ref(s) },
        }
    }
    pub unsafe fn from_mut(s: &mut [u8]) -> IoVec {
        IoVec {
            inner: unsafe { sys::IoVec::from_mut(s) },
        }
    }

    pub unsafe fn from_uninit_mut(s: &mut [MaybeUninit<u8>]) -> IoVec {
        IoVec {
            inner: unsafe { sys::IoVec::from_uninit_mut(s) },
        }
    }
}
