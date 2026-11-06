pub(crate) mod io;

pub(crate) mod net;

use core::mem::MaybeUninit;

#[repr(transparent)]
#[derive(Clone)]
struct Padding<T: Copy>(MaybeUninit<T>);

impl<T: Copy> Default for Padding<T> {
    fn default() -> Self {
        Self(MaybeUninit::zeroed())
    }
}
