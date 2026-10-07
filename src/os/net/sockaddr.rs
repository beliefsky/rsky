use std::{
    mem::{self, MaybeUninit},
    net::{SocketAddr, SocketAddrV4, SocketAddrV6},
    ptr,
};

use super::AddressFamily;

#[derive(Clone)]
pub struct SockAddr {
    storage: sys::sockaddr_storage,
    len: u32,
}

impl SockAddr {
    pub fn family(&self) -> AddressFamily {
        AddressFamily::from_raw(self.storage.ss_family)
    }

    pub fn len(&self) -> u32 {
        self.len
    }

    /// Returns a raw pointer to the address.
    pub fn addr_as_ptr<T>(&self) -> *const T {
        &self.storage as *const sys::sockaddr_storage as _
    }

    pub fn buffer_ptr_mut<T>(&mut self) -> (*mut T, *mut u32) {
        (
            &mut self.storage as *mut sys::sockaddr_storage as _,
            &mut self.len,
        )
    }
}

impl Default for SockAddr {
    fn default() -> Self {
        SockAddr {
            storage: unsafe { mem::zeroed() },
            len: mem::size_of::<sys::sockaddr_storage>() as _,
        }
    }
}

impl From<SocketAddr> for SockAddr {
    fn from(addr: SocketAddr) -> SockAddr {
        match addr {
            SocketAddr::V4(addr) => addr.into(),
            SocketAddr::V6(addr) => addr.into(),
        }
    }
}

impl From<SocketAddrV4> for SockAddr {
    fn from(addr: SocketAddrV4) -> Self {
        let mut storage = unsafe { mem::zeroed::<sys::sockaddr_storage>() };
        let len = {
            let storage = unsafe { &mut *ptr::addr_of_mut!(storage).cast::<sys::sockaddr_in>() };
            storage.sin_family = AddressFamily::INET.as_raw();
            storage.sin_port = addr.port().to_be();
            storage.sin_addr.s_addr = u32::from_ne_bytes(addr.ip().octets());
            storage.sin_zero = Default::default();

            mem::size_of::<sys::sockaddr_in>() as _
        };

        SockAddr { storage, len }
    }
}

impl From<SocketAddrV6> for SockAddr {
    fn from(addr: SocketAddrV6) -> SockAddr {
        let mut storage = unsafe { mem::zeroed::<sys::sockaddr_storage>() };
        let len = {
            let storage = unsafe { &mut *ptr::addr_of_mut!(storage).cast::<sys::sockaddr_in6>() };
            storage.sin6_family = AddressFamily::INET6.as_raw();
            storage.sin6_port = addr.port().to_be();
            storage.sin6_addr.s6_addr = addr.ip().octets();
            storage.sin6_flowinfo = addr.flowinfo();
            storage.sin6_scope_id = addr.scope_id();

            mem::size_of::<sys::sockaddr_in6>() as _
        };
        SockAddr { storage, len }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
struct Padding<T: Copy>(MaybeUninit<T>);

impl<T: Copy> Default for Padding<T> {
    fn default() -> Self {
        Self(MaybeUninit::zeroed())
    }
}

impl<T: Copy> Padding<T> {
    pub(crate) const fn new(val: T) -> Self {
        Self(MaybeUninit::new(val))
    }
}

#[cfg(target_os = "linux")]
mod sys {
    use super::Padding;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub(super) struct sockaddr_storage {
        pub ss_family: u16,
        #[cfg(target_pointer_width = "32")]
        __ss_pad2: Padding<[u8; 128 - 2 - 4]>,
        #[cfg(target_pointer_width = "64")]
        __ss_pad2: Padding<[u8; 128 - 2 - 8]>,
        __ss_align: usize,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub(super) struct sockaddr_in {
        pub sin_family: u16,
        pub sin_port: u16,
        pub sin_addr: in_addr,
        pub sin_zero: [u8; 8],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub(super) struct sockaddr_in6 {
        pub sin6_family: u16,
        pub sin6_port: u16,
        pub sin6_flowinfo: u32,
        pub sin6_addr: in6_addr,
        pub sin6_scope_id: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub(super) struct in_addr {
        pub s_addr: u32,
    }

    #[repr(C)]
    #[repr(align(4))]
    #[derive(Clone, Copy)]
    pub(super) struct in6_addr {
        pub s6_addr: [u8; 16],
    }
}
