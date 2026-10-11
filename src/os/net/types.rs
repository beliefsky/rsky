use super::super::sys;

pub type RawAddressFamily = sys::SaFamily;

pub const SOCK_CLOEXEC: i32 = sys::O_CLOEXEC;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct AddressFamily(RawAddressFamily);

impl AddressFamily {
    pub const UNSPEC: Self = Self(sys::AF_UNSPEC as _);

    pub const INET: Self = Self(sys::AF_INET as _);

    pub const INET6: Self = Self(sys::AF_INET6 as _);

    #[inline]
    pub const fn from_raw(raw: RawAddressFamily) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn as_raw(self) -> RawAddressFamily {
        self.0
    }
}

pub type RawSocketType = u32;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct SocketType(RawSocketType);
impl SocketType {
    pub const STREAM: Self = Self(sys::SOCK_STREAM as _);

    pub const DGRAM: Self = Self(sys::SOCK_DGRAM as _);

    #[inline]
    pub const fn from_raw(raw: RawSocketType) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn as_raw(self) -> RawSocketType {
        self.0
    }
}

pub type RawProtocol = core::num::NonZeroU32;

const fn new_raw_protocol(u: u32) -> RawProtocol {
    match RawProtocol::new(u) {
        Some(p) => p,
        None => panic!("new_raw_protocol: protocol must be non-zero"),
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Protocol(pub(crate) RawProtocol);

impl Protocol {
    pub const TCP: Protocol = Protocol(new_raw_protocol(sys::IPPROTO_TCP as _));

    pub const UDP: Protocol = Protocol(new_raw_protocol(sys::IPPROTO_UDP as _));

    /// Constructs a `Protocol` from a raw integer.
    #[inline]
    pub const fn from_raw(raw: RawProtocol) -> Self {
        Self(raw)
    }

    /// Returns the raw integer for this `Protocol`.
    #[inline]
    pub const fn as_raw(self) -> RawProtocol {
        self.0
    }
}
