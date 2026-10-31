use core::{ffi::c_void, mem::MaybeUninit};

pub(crate) const IOVEC_MAX_LEN: usize = usize::MAX;

#[repr(C)]
#[derive(Clone)]
pub(crate) struct IoVec {
    buf: *mut c_void,
    len: usize,
}

impl IoVec {
    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) unsafe fn from_ref(s: &[u8]) -> Self {
        Self {
            buf: s.as_ptr() as _,
            len: s.len(),
        }
    }

    pub(crate) unsafe fn from_mut(s: &mut [u8]) -> Self {
        Self {
            buf: s.as_mut_ptr() as _,
            len: s.len(),
        }
    }

    pub(crate) unsafe fn from_uninit_mut(s: &mut [MaybeUninit<u8>]) -> Self {
        Self {
            buf: s.as_mut_ptr() as _,
            len: s.len(),
        }
    }
}
