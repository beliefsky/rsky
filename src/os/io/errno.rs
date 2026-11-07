use core::{error::Error, fmt, result};

pub use crate::{io::ErrorKind, os::sys::io::errno::Errno};

pub type Result<T> = result::Result<T, Errno>;

impl Errno {
    pub fn kind(self) -> ErrorKind {
        match self {
            Self::EADDRINUSE => ErrorKind::AddrInUse,
            Self::EADDRNOTAVAIL => ErrorKind::AddrNotAvailable,
            Self::EPIPE => ErrorKind::BrokenPipe,
            Self::ECONNABORTED => ErrorKind::ConnectionAborted,
            Self::ECONNREFUSED => ErrorKind::ConnectionRefused,
            Self::ECONNRESET => ErrorKind::ConnectionReset,
            Self::EXDEV => ErrorKind::CrossesDevices,
            Self::ENOTEMPTY => ErrorKind::DirectoryNotEmpty,
            Self::EINTR => ErrorKind::Interrupted,
            Self::EFBIG => ErrorKind::FileTooLarge,
            Self::EINVAL => ErrorKind::InvalidInput,
            Self::EISDIR => ErrorKind::IsADirectory,
            Self::ENOTDIR => ErrorKind::NotADirectory,
            Self::ENOTCONN => ErrorKind::NotConnected,
            Self::ENOENT => ErrorKind::NotFound,
            Self::ESPIPE => ErrorKind::NotSeekable,
            Self::EACCES => ErrorKind::PermissionDenied,
            Self::EBUSY => ErrorKind::ResourceBusy,
            Self::ETIME => ErrorKind::TimedOut,
            Self::EMLINK => ErrorKind::TooManyLinks,
            Self::EMFILE => ErrorKind::TooManyOpenFiles,
            Self::EAGAIN | Self::EWOULDBLOCK => ErrorKind::WouldBlock,

            _ => {
                println!("errno unknown: {}", self);
                ErrorKind::Other
            }
        }
    }
}

impl fmt::Display for Errno {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(errno: {}, message: {})",
            self.raw_os_error(),
            self.errno_msg()
        )
    }
}

impl fmt::Debug for Errno {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Errno(errno: {}, message: {})",
            self.raw_os_error(),
            self.errno_msg()
        )
    }
}

impl Error for Errno {}
