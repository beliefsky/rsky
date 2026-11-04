use crate::os::error::{os_error_kind, os_error_msg};
use core::{error, fmt};

#[repr(transparent)]
pub struct Error {
    inner: Inner,
}

impl Error {
    pub fn errno(errno: u32) -> Self {
        Self {
            inner: Inner::Os(errno),
        }
    }

    pub fn simple(kind: ErrorKind) -> Self {
        Self {
            inner: Inner::Simple(kind),
        }
    }

    pub fn simple_message(kind: ErrorKind, msg: &'static str) -> Self {
        Self {
            inner: Inner::SimpleMessage((kind, msg)),
        }
    }

    pub fn other<T: error::Error + 'static>(error: T) -> Self {
        Self {
            inner: Inner::Other(Box::new(error)),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        match self.inner {
            Inner::Os(errno) => os_error_kind(errno),
            Inner::Simple(kind) => kind,
            Inner::SimpleMessage(v) => v.0,
            _ => ErrorKind::Other,
        }
    }

    pub fn as_errno(&self) -> Option<u32> {
        if let Inner::Os(e) = self.inner {
            Some(e)
        } else {
            None
        }
    }
    pub fn as_other(&self) -> Option<&dyn error::Error> {
        if let Inner::Other(e) = &self.inner {
            Some(e.as_ref())
        } else {
            None
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            Inner::Os(errno) => write!(f, "{}", os_error_msg(*errno)),
            Inner::Simple(e) => write!(f, "{}", e.as_str()),
            Inner::SimpleMessage(v) => write!(f, "{}", v.1),
            Inner::Other(e) => fmt::Debug::fmt(e, f),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            Inner::Os(errno) => write!(f, "{}", os_error_msg(*errno)),
            Inner::Simple(e) => write!(f, "{}", e.as_str()),
            Inner::SimpleMessage(v) => write!(f, "{}", v.1),
            Inner::Other(e) => fmt::Display::fmt(e, f),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    HostUnreachable,
    NetworkUnreachable,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    NetworkDown,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    NotADirectory,
    IsADirectory,
    DirectoryNotEmpty,
    ReadOnlyFilesystem,
    FilesystemLoop,
    StaleNetworkFileHandle,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    StorageFull,
    NotSeekable,
    QuotaExceeded,
    FileTooLarge,
    ResourceBusy,
    ExecutableFileBusy,
    Deadlock,
    CrossesDevices,
    TooManyLinks,
    InvalidFilename,
    ArgumentListTooLong,
    Interrupted,
    Unsupported,
    UnexpectedEof,
    OutOfMemory,
    InProgress,
    TooManyOpenFiles,
    InputOutputError,
    Other,
    Uncategorized,
}

impl ErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AddrInUse => "address in use",
            Self::AddrNotAvailable => "address not available",
            Self::AlreadyExists => "entity already exists",
            Self::ArgumentListTooLong => "argument list too long",
            Self::BrokenPipe => "broken pipe",
            Self::ConnectionAborted => "connection aborted",
            Self::ConnectionRefused => "connection refused",
            Self::ConnectionReset => "connection reset",
            Self::CrossesDevices => "cross-device link or rename",
            Self::Deadlock => "deadlock",
            Self::DirectoryNotEmpty => "directory not empty",
            Self::ExecutableFileBusy => "executable file busy",
            Self::FileTooLarge => "file too large",
            Self::FilesystemLoop => "filesystem loop or indirection limit (e.g. symlink loop)",
            Self::HostUnreachable => "host unreachable",
            Self::InProgress => "in progress",
            Self::InputOutputError => "input/output error",
            Self::Interrupted => "operation interrupted",
            Self::InvalidData => "invalid data",
            Self::InvalidFilename => "invalid filename",
            Self::InvalidInput => "invalid input parameter",
            Self::IsADirectory => "is a directory",
            Self::NetworkDown => "network down",
            Self::NetworkUnreachable => "network unreachable",
            Self::NotADirectory => "not a directory",
            Self::NotConnected => "not connected",
            Self::NotFound => "entity not found",
            Self::NotSeekable => "seek on unseekable file",
            Self::Other => "other error",
            Self::OutOfMemory => "out of memory",
            Self::PermissionDenied => "permission denied",
            Self::QuotaExceeded => "quota exceeded",
            Self::ReadOnlyFilesystem => "read-only filesystem or storage medium",
            Self::ResourceBusy => "resource busy",
            Self::StaleNetworkFileHandle => "stale network file handle",
            Self::StorageFull => "no storage space",
            Self::TimedOut => "timed out",
            Self::TooManyLinks => "too many links",
            Self::TooManyOpenFiles => "too many open files",
            Self::Uncategorized => "uncategorized error",
            Self::UnexpectedEof => "unexpected end of file",
            Self::Unsupported => "unsupported",
            Self::WouldBlock => "operation would block",
            Self::WriteZero => "write zero",
        }
    }
}

enum Inner {
    Os(u32),
    Simple(ErrorKind),
    SimpleMessage((ErrorKind, &'static str)),
    Other(Box<dyn error::Error>),
}
