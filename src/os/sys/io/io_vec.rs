use core::{mem::MaybeUninit, ptr};

impl IoVec {
    pub fn null() -> Self {
        Self {
            buf: ptr::null_mut(),
            len: 0,
        }
    }

    #[inline]
    pub(crate) fn from_slice(slice: &[u8]) -> Self {
        assert!(slice.len() < IOVEC_MAX_LEN);
        Self {
            buf: slice.as_ptr() as _,
            len: slice.len() as _,
        }
    }

    #[inline]
    pub(crate) fn from_uninit(slice: &mut [MaybeUninit<u8>]) -> Self {
        assert!(slice.len() < IOVEC_MAX_LEN);
        Self {
            buf: slice.as_ptr() as _,
            len: slice.len() as _,
        }
    }
}

impl Default for IoVec {
    fn default() -> Self {
        Self::null()
    }
}

#[cfg(unix)]
pub(crate) use unix::{IOVEC_MAX_LEN, IoVec};

#[cfg(unix)]
mod unix {
    use core::ffi::c_void;

    pub(crate) const IOVEC_MAX_LEN: usize = usize::MAX;

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub(crate) struct IoVec {
        pub(crate) buf: *mut c_void,
        pub(crate) len: usize,
    }
}

#[cfg(windows)]
pub(crate) use windows::{IOVEC_MAX_LEN, IoVec};

#[cfg(windows)]
mod windows {
    use core::ffi::{c_char, c_ulong};

    pub(crate) const IOVEC_MAX_LEN: usize = c_ulong::MAX as _;

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub(crate) struct IoVec {
        pub(crate) len: c_ulong,
        pub(crate) buf: *mut c_char,
    }
}
