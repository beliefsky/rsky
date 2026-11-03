use core::{ffi::c_void, mem};

use super::super::{IoVec, Socklen};

#[repr(C)]
#[derive(Clone)]
pub(crate) struct MsgHdr {
    msg_name: *mut c_void,
    msg_namelen: Socklen,
    msg_iov: *mut IoVec,
    msg_iovlen: usize,
    msg_control: *mut c_void,
    msg_controllen: usize,
    msg_flags: i32,
}

impl MsgHdr {
    #[inline]
    pub(crate) fn new() -> Self {
        unsafe { mem::zeroed() }
    }

    #[inline]
    pub fn set_vec_ref(&mut self, vec: &[IoVec]) {
        self.msg_iov = vec.as_ptr() as _;
        self.msg_iovlen = vec.len();
    }

    #[inline]
    pub fn set_vec_mut(&mut self, vec: &mut [IoVec]) {
        self.msg_iov = vec.as_mut_ptr();
        self.msg_iovlen = vec.len();
    }
}
