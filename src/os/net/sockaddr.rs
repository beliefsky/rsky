use std::{
    mem,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    ptr,
};

use super::{super::sys, AddressFamily};

#[derive(Clone)]
pub struct SockAddr {
    storage: sys::sockaddr_storage,
    len: sys::Socklen,
}

impl SockAddr {
    pub fn family(&self) -> AddressFamily {
        AddressFamily::from_raw(self.storage.ss_family)
    }

    pub fn len(&self) -> sys::Socklen {
        self.len
    }

    /// Returns a raw pointer to the address.
    pub fn as_ptr<T>(&self) -> *const T {
        &self.storage as *const sys::sockaddr_storage as _
    }

    pub fn as_ptr_len_mut<T>(&mut self) -> (*mut T, *mut sys::Socklen) {
        (
            &mut self.storage as *mut sys::sockaddr_storage as _,
            &mut self.len,
        )
    }

    pub fn as_addr(&self) -> Option<SocketAddr> {
        let addr = match self.family() {
            AddressFamily::INET => {
                let storage = unsafe { &(*self.as_ptr::<sys::sockaddr_in>()) };

                SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::from(storage.sin_addr.s_addr.to_ne_bytes()),
                    storage.sin_port.to_be(),
                ))
            }
            AddressFamily::INET6 => {
                let storage = unsafe { &(*self.as_ptr::<sys::sockaddr_in6>()) };
                SocketAddr::V6(SocketAddrV6::new(
                    Ipv6Addr::from(storage.sin6_addr.s6_addr),
                    storage.sin6_port.to_be(),
                    storage.sin6_flowinfo,
                    storage.sin6_scope_id,
                ))
            }
            _ => {
                return None;
            }
        };
        Some(addr)
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
