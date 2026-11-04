mod error;
mod iovec;
mod msghdr;
mod sockaddr;
mod types;

pub(crate) use error::*;
pub(crate) use iovec::*;
pub(crate) use msghdr::*;
pub(crate) use sockaddr::*;
pub(crate) use types::*;

use core::mem::MaybeUninit;

#[repr(transparent)]
#[derive(Clone)]
struct Padding<T: Copy>(MaybeUninit<T>);

impl<T: Copy> Default for Padding<T> {
    fn default() -> Self {
        Self(MaybeUninit::zeroed())
    }
}
