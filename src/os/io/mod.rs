mod errno;

pub use errno::{Errno, Result};

pub(crate) use crate::os::sys::io::io_vec::IoVec;
