use core::{ffi::c_void, ptr};

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
    pub(crate) unsafe fn from_vec_ref(iovec: &[IoVec]) -> Self {
        Self {
            msg_name: ptr::null_mut(),
            msg_namelen: 0,
            msg_iov: iovec.as_ptr() as _,
            msg_iovlen: iovec.len(),
            msg_control: ptr::null_mut(),
            msg_controllen: 0,
            msg_flags: 0,
        }
    }
    pub(crate) unsafe fn from_vec_mut(iovec: &mut [IoVec]) -> Self {
        Self {
            msg_name: ptr::null_mut(),
            msg_namelen: 0,
            msg_iov: iovec.as_mut_ptr(),
            msg_iovlen: iovec.len(),
            msg_control: ptr::null_mut(),
            msg_controllen: 0,
            msg_flags: 0,
        }
    }
}
