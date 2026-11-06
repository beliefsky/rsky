#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "linux")]
mod linux {
    use crate::os::sys::{Padding, net::SaFamily};

    #[repr(C)]
    #[derive(Clone)]
    pub struct SocketAddrStorage {
        pub(crate) ss_family: SaFamily,
        #[cfg(target_pointer_width = "32")]
        __ss_pad2: Padding<[u8; 128 - 2 - 4]>,
        #[cfg(target_pointer_width = "64")]
        __ss_pad2: Padding<[u8; 128 - 2 - 8]>,
        __ss_align: usize,
    }

    #[repr(C)]
    #[derive(Clone)]
    pub(crate) struct SocketAddrIn {
        pub(crate) sin_family: SaFamily,
        pub(crate) sin_port: u16,
        pub(crate) sin_addr: InAddr,
        sin_zero: [u8; 8],
    }

    #[repr(C)]
    #[derive(Clone)]
    pub(crate) struct SocketAddrIn6 {
        pub(crate) sin6_family: SaFamily,
        pub(crate) sin6_port: u16,
        pub(crate) sin6_flowinfo: u32,
        pub(crate) sin6_addr: In6Addr,
        pub(crate) sin6_scope_id: u32,
    }

    #[repr(C)]
    #[derive(Clone)]
    pub(crate) struct InAddr {
        pub(crate) s_addr: u32,
    }

    #[repr(C)]
    #[repr(align(4))]
    #[derive(Clone)]
    pub(crate) struct In6Addr {
        pub(crate) s6_addr: [u8; 16],
    }
}
