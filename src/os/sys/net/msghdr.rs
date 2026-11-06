use core::{ffi::c_void, mem};

use crate::os::sys::{io::io_vec::IoVec, net::Socklen};

#[repr(C)]
#[derive(Clone)]
pub(crate) struct Msghdr {
    msg_name: *mut c_void,
    msg_namelen: Socklen,
    msg_iov: *mut IoVec,
    msg_iovlen: usize,
    msg_control: *mut c_void,
    msg_controllen: usize,
    msg_flags: i32,
}

impl Msghdr {
    #[inline]
    pub(crate) fn set_ref_vec(&mut self, vec: &[IoVec]) {
        self.msg_iov = vec.as_ptr() as _;
        self.msg_iovlen = vec.len();
    }

    #[inline]
    pub(crate) fn set_mut_vec(&mut self, vec: &mut [IoVec]) {
        self.msg_iov = vec.as_mut_ptr();
        self.msg_iovlen = vec.len();
    }

    #[inline]
    pub fn as_ptr(&self) -> *const Msghdr {
        self
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut Msghdr {
        self
    }
}

impl Default for Msghdr {
    fn default() -> Self {
        unsafe { mem::zeroed() }
    }
}
