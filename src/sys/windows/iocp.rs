

use super::types::{self, BOOL, HANDLE};


  extern "C" {
    fn CreateIoCompletionPort(
        filehandle: HANDLE,
        existingcompletionport: HANDLE,
        completionkey: usize,
        numberofconcurrentthreads: u32
    ) -> HANDLE;

    fn GetQueuedCompletionStatus(
        completionport: HANDLE,
        lpnumberofbytestransferred: *mut u32,
        lpCompletionKey: *mut usize,
        lpoverlapped: *mut OVERLAPPED,
        dwmilliseconds: u32
    ) -> BOOL;

    fn CancelIo(hfile: HANDLE) -> BOOL;


    fn CloseHandle(handle: HANDLE) -> BOOL;

    fn GetLastError() ->u32;
}

#[inline]
pub(crate) unsafe fn createiocompletionport(
    filehandle: HANDLE,
    existingcompletionport: HANDLE,
    completionkey: usize,
    numberofconcurrentthreads: u32
) -> Result<HANDLE, &'static str> {
    let h = CreateIoCompletionPort(filehandle, existingcompletionport, completionkey, numberofconcurrentthreads);

    if !(h == types::INVALID_HANDLE_VALUE) {
        Result::Ok(h)
    } else {
        Result::Err("CreateIoCompletionPort fail")
    }
}

#[inline]
pub(crate) unsafe fn cancelio(hfile: HANDLE) -> bool {

    CancelIo(hfile) == types::TRUE
}

#[inline]
pub(crate) unsafe fn closehandle(hfile: HANDLE) -> bool {
    CloseHandle(hfile) == types::TRUE
}


#[repr(C)]
pub struct OVERLAPPED {
    pub internal: usize,
    pub internal_high: usize,
    pub anonymous: OVERLAPPED_0,
    pub h_event: HANDLE
}

impl ::core::marker::Copy for OVERLAPPED {}
impl ::core::clone::Clone for OVERLAPPED {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::core::default::Default for OVERLAPPED {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}

#[repr(C)]
pub union OVERLAPPED_0 {
    pub anonymous: OVERLAPPED_0_0,
    pub pointer: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for OVERLAPPED_0 {}
impl ::core::clone::Clone for OVERLAPPED_0 {
    fn clone(&self) -> Self {
        *self
    }
}

impl ::core::default::Default for OVERLAPPED_0 {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}

#[repr(C)]
pub struct OVERLAPPED_0_0 {
    pub offset: u32,
    pub offset_high: u32,
}
impl ::core::marker::Copy for OVERLAPPED_0_0 {}
impl ::core::clone::Clone for OVERLAPPED_0_0 {
    fn clone(&self) -> Self {
        *self
    }
}

impl ::core::cmp::PartialEq for OVERLAPPED_0_0 {
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset && self.offset_high == other.offset_high
    }
}
impl ::core::cmp::Eq for OVERLAPPED_0_0 {}
impl ::core::default::Default for OVERLAPPED_0_0 {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}