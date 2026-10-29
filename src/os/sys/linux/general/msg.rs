use core::ffi::c_void;

use super::Socklen;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Msghdr {
    pub msg_name: *mut c_void,
    pub msg_namelen: Socklen,
    pub msg_iov: *mut Iovec,
    pub msg_iovlen: usize,
    pub msg_control: *mut c_void,
    pub msg_controllen: usize,
    pub msg_flags: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Iovec {
    pub iov_base: *mut c_void,
    pub iov_len: usize,
}
