use super::super::{io::IoVec, sys};

pub struct RecvFlags(u32);

impl RecvFlags {
    pub const EMPTY: Self = Self(0);
    pub const PEEK: Self = Self(sys::MSG_PEEK);

    #[inline]
    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    #[inline]
    pub fn flags(&self) -> u32 {
        self.0
    }
}

pub struct SendFlags(u32);

impl SendFlags {
    pub const EMPTY: Self = Self(0);
    pub const MSG_NOSIGNAL: Self = Self(sys::MSG_NOSIGNAL);
    pub const MSG_OOB: Self = Self(sys::MSG_OOB);

    #[inline]
    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    #[inline]
    pub fn flags(&self) -> u32 {
        self.0
    }
}

pub struct Shutdown(i32);

impl Shutdown {
    pub const READ: Self = Self(sys::SHUT_RD);
    pub const WRITE: Self = Self(sys::SHUT_WR);
    pub const BOTH: Self = Self(sys::SHUT_RDWR);

    #[inline]
    pub fn how(&self) -> i32 {
        self.0
    }
}

pub struct MsgHdr {
    _inner: sys::MsgHdr,
}

impl MsgHdr {
    pub unsafe fn from_vec_ref(iovec: &[IoVec]) -> Self {
        let iovec = unsafe { &*(iovec as *const [IoVec] as *const [sys::IoVec]) };
        MsgHdr {
            _inner: unsafe { sys::MsgHdr::from_vec_ref(iovec) },
        }
    }

    pub unsafe fn from_vec_mut(iovec: &mut [IoVec]) -> Self {
        let iovec = unsafe { &mut *(iovec as *mut [IoVec] as *mut [sys::IoVec]) };
        MsgHdr {
            _inner: unsafe { sys::MsgHdr::from_vec_mut(iovec) },
        }
    }

    pub fn as_ptr(&self) -> *const MsgHdr {
        self as _
    }
    pub fn as_ptr_mut(&mut self) -> *mut MsgHdr {
        self as _
    }
}
