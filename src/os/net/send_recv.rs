use super::super::sys;
use std::mem;

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

#[repr(transparent)]
pub struct IoVec(sys::Iovec);

impl IoVec {
    #[inline]
    pub fn from_slice(value: &[u8]) -> Self {
        Self(sys::Iovec {
            iov_base: value.as_ptr() as _,
            iov_len: value.len() as _,
        })
    }

    #[inline]
    pub fn from_uninit(value: &mut [std::mem::MaybeUninit<u8>]) -> Self {
        Self(sys::Iovec {
            iov_base: value.as_mut_ptr() as _,
            iov_len: value.len() as _,
        })
    }

    #[inline]
    pub fn ptr(&self) -> *mut std::mem::MaybeUninit<u8> {
        return self.0.iov_base as *mut std::mem::MaybeUninit<u8>;
    }

    #[inline]
    pub fn len(&self) -> usize {
        return self.0.iov_len;
    }
}

impl Default for IoVec {
    fn default() -> Self {
        Self(unsafe { mem::zeroed() })
    }
}

#[repr(transparent)]
pub struct MsgHdr(sys::Msghdr);

impl MsgHdr {
    #[inline]
    pub fn set_vec(&mut self, value: &[IoVec]) {
        self.0.msg_iov = value.as_ptr() as _;
        self.0.msg_iovlen = value.len();
    }

    #[inline]
    pub fn set_mut_vec(&mut self, value: &mut [IoVec]) {
        self.0.msg_iov = value.as_mut_ptr() as _;
        self.0.msg_iovlen = value.len();
    }

    #[inline]
    pub fn as_ptr<T>(&self) -> *const T {
        &self.0 as *const sys::Msghdr as _
    }

    #[inline]
    pub fn as_mut_ptr<T>(&mut self) -> *mut T {
        &mut self.0 as *mut sys::Msghdr as _
    }
}

impl Default for MsgHdr {
    fn default() -> Self {
        Self(unsafe { mem::zeroed() })
    }
}
