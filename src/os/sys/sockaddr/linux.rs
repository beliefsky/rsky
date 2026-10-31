use super::super::{Padding, SaFamily};

#[repr(C)]
#[derive(Clone)]
pub struct sockaddr_storage {
    pub(crate) ss_family: SaFamily,
    #[cfg(target_pointer_width = "32")]
    __ss_pad2: Padding<[u8; 128 - 2 - 4]>,
    #[cfg(target_pointer_width = "64")]
    __ss_pad2: Padding<[u8; 128 - 2 - 8]>,
    __ss_align: usize,
}

#[repr(C)]
#[derive(Clone)]
pub(crate) struct sockaddr_in {
    pub(crate) sin_family: SaFamily,
    pub(crate) sin_port: u16,
    pub(crate) sin_addr: in_addr,
    sin_zero: [u8; 8],
}

#[repr(C)]
#[derive(Clone)]
pub(crate) struct sockaddr_in6 {
    pub(crate) sin6_family: SaFamily,
    pub(crate) sin6_port: u16,
    pub(crate) sin6_flowinfo: u32,
    pub(crate) sin6_addr: in6_addr,
    pub(crate) sin6_scope_id: u32,
}

#[repr(C)]
#[derive(Clone)]
pub(crate) struct in_addr {
    pub(crate) s_addr: u32,
}

#[repr(C)]
#[repr(align(4))]
#[derive(Clone)]
pub(crate) struct in6_addr {
    pub(crate) s6_addr: [u8; 16],
}
