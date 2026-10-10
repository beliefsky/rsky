use super::super::sys;

pub struct RecvFlags(u32);

impl RecvFlags {
    pub const EMPTY: Self = Self(0);
    pub const PEEK: Self = Self(sys::MSG_PEEK);

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn flags(&self) -> u32 {
        self.0
    }
}

pub struct SendFlags(u32);

impl SendFlags {
    pub const EMPTY: Self = Self(0);
    pub const MSG_NOSIGNAL: Self = Self(sys::MSG_NOSIGNAL);
    pub const MSG_OOB: Self = Self(sys::MSG_OOB);

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn flags(&self) -> u32 {
        self.0
    }
}

pub struct Shutdown(i32);

impl Shutdown {
    pub const READ: Self = Self(sys::SHUT_RD);
    pub const WRITE: Self = Self(sys::SHUT_WR);
    pub const BOTH: Self = Self(sys::SHUT_RDWR);

    pub fn how(&self) -> i32 {
        self.0
    }
}
