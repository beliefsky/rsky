use crate::os::sys::net;

#[repr(transparent)]
pub struct RecvFlags(u32);

impl RecvFlags {
    pub const EMPTY: Self = Self(0);
    pub const PEEK: Self = Self(net::MSG_PEEK);

    #[inline]
    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    #[inline]
    pub fn flags(&self) -> u32 {
        self.0
    }
}

#[repr(transparent)]
pub struct SendFlags(u32);

impl SendFlags {
    pub const EMPTY: Self = Self(0);
    pub const MSG_NOSIGNAL: Self = Self(net::MSG_NOSIGNAL);
    pub const MSG_OOB: Self = Self(net::MSG_OOB);

    #[inline]
    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    #[inline]
    pub fn flags(&self) -> u32 {
        self.0
    }
}

#[repr(transparent)]
pub struct Shutdown(i32);

impl Shutdown {
    pub const READ: Self = Self(net::SHUT_RD);
    pub const WRITE: Self = Self(net::SHUT_WR);
    pub const BOTH: Self = Self(net::SHUT_RDWR);

    #[inline]
    pub fn how(&self) -> i32 {
        self.0
    }
}
