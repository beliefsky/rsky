use std::{
    io,
    ops::Deref,
    os::fd::{AsFd, AsRawFd},
};

use crate::{driver::SharedFd, io::IntoInner, runtime::Runtime};

pub(crate) struct Attacher<S> {
    source: SharedFd<S>,
}

impl<S> Attacher<S> {
    unsafe fn new_unchecked(source: S) -> Self {
        Self {
            source: unsafe { SharedFd::new_unchecked(source) },
        }
    }
}

impl<S: AsFd> Attacher<S> {
    pub fn new(source: S) -> io::Result<Self> {
        Runtime::with_current(|r| r.attach(source.as_fd().as_raw_fd()))?;
        Ok(unsafe { Self::new_unchecked(source) })
    }

    pub fn shared_fd(&self) -> SharedFd<S> {
        self.source.clone()
    }
}

impl<S> Clone for Attacher<S> {
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
        }
    }
}

impl<S> Deref for Attacher<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        self.source.deref()
    }
}

impl<S> IntoInner for Attacher<S> {
    type Inner = SharedFd<S>;

    fn into_inner(self) -> Self::Inner {
        self.source
    }
}
