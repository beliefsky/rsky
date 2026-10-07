use std::{mem::ManuallyDrop, os::fd::OwnedFd};

use crate::{
    buf::IntoInner,
    os::net::{AddressFamily, Protocol, SockAddr, SocketType},
};

mod iour;

pub struct CreateSocket {
    pub(crate) domain: AddressFamily,
    pub(crate) socket_type: SocketType,
    pub(crate) protocol: Option<Protocol>,
    pub(crate) opened_fd: Option<OwnedFd>,
}

impl CreateSocket {
    pub fn new(domain: AddressFamily, socket_type: SocketType, protocol: Option<Protocol>) -> Self {
        CreateSocket {
            domain,
            socket_type,
            protocol,
            opened_fd: None,
        }
    }
}

impl IntoInner for CreateSocket {
    type Inner = OwnedFd;

    fn into_inner(self) -> Self::Inner {
        self.opened_fd.expect("socket not created")
    }
}

pub struct Bind<S> {
    pub(crate) fd: S,
    pub(crate) addr: SockAddr,
}

impl<S> Bind<S> {
    pub fn new(fd: S, addr: SockAddr) -> Self {
        Self { fd, addr }
    }
}

pub struct Listen<S> {
    pub(crate) fd: S,
    pub(crate) backlog: i32,
}

impl<S> Listen<S> {
    pub fn new(fd: S, backlog: i32) -> Self {
        Self { fd, backlog }
    }
}

pub struct Accept<S> {
    pub(crate) fd: S,
    pub(crate) addr: SockAddr,
    pub(crate) accepted_fd: Option<OwnedFd>,
}

impl<S> Accept<S> {
    /// Create [`Accept`].
    pub fn new(fd: S) -> Self {
        Self {
            fd,
            addr: SockAddr::default(),
            accepted_fd: None,
        }
    }
}

impl<S> IntoInner for Accept<S> {
    type Inner = (OwnedFd, SockAddr);

    fn into_inner(mut self) -> Self::Inner {
        let socket = self.accepted_fd.take().expect("socket not accepted");
        (socket, self.addr)
    }
}

pub struct Connect<S> {
    pub(crate) fd: S,
    pub(crate) addr: SockAddr,
}

impl<S> Connect<S> {
    pub fn new(fd: S, addr: SockAddr) -> Self {
        Self { fd, addr }
    }
}

pub struct CloseSocket {
    pub(crate) fd: ManuallyDrop<OwnedFd>,
}

impl CloseSocket {
    pub fn new(fd: OwnedFd) -> Self {
        Self {
            fd: ManuallyDrop::new(fd),
        }
    }
}
