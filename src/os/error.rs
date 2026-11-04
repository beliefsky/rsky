use super::sys::{self, sys_errno_msg};
use crate::io::error::ErrorKind;

pub fn os_error_kind(errno: u32) -> ErrorKind {
    match errno {
        sys::EADDRINUSE => ErrorKind::AddrInUse,
        sys::EADDRNOTAVAIL => ErrorKind::AddrNotAvailable,
        sys::EPIPE => ErrorKind::BrokenPipe,
        sys::ECONNABORTED => ErrorKind::ConnectionAborted,
        sys::ECONNREFUSED => ErrorKind::ConnectionRefused,
        sys::ECONNRESET => ErrorKind::ConnectionReset,
        sys::EXDEV => ErrorKind::CrossesDevices,
        sys::ENOTEMPTY => ErrorKind::DirectoryNotEmpty,
        sys::EAGAIN | sys::EWOULDBLOCK => ErrorKind::Interrupted,
        sys::EFBIG => ErrorKind::FileTooLarge,
        sys::EINVAL => ErrorKind::InvalidInput,
        sys::EISDIR => ErrorKind::IsADirectory,
        sys::ENOTDIR => ErrorKind::NotADirectory,
        sys::ENOTCONN => ErrorKind::NotConnected,
        sys::ENOENT => ErrorKind::NotFound,
        sys::ESPIPE => ErrorKind::NotSeekable,
        sys::EACCES => ErrorKind::PermissionDenied,
        sys::EBUSY => ErrorKind::ResourceBusy,
        sys::ETIME => ErrorKind::TimedOut,
        sys::EMLINK => ErrorKind::TooManyLinks,
        sys::EMFILE => ErrorKind::TooManyOpenFiles,

        _ => {
            println!("errno unknown: {}", errno);
            ErrorKind::Other
        }
    }
}

pub fn os_error_msg(errno: u32) -> &'static str {
    sys_errno_msg(errno)
}
