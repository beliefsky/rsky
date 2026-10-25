use std::{mem::ManuallyDrop, os::fd::OwnedFd};

use crate::{
    io::{IntoInner, IoBuf, IoBufMut, IoVectoredBuf, IoVectoredBufMut},
    os::net::{
        AddressFamily, IoVec, MsgHdr, Protocol, RecvFlags, SendFlags, Shutdown, SockAddr,
        SocketType,
    },
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

pub struct ShutdownSocket<S> {
    pub(crate) fd: S,
    pub(crate) how: Shutdown,
}

impl<S> ShutdownSocket<S> {
    pub fn new(fd: S, how: Shutdown) -> Self {
        Self { fd, how }
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

pub struct Recv<T: IoBufMut, S> {
    pub(crate) fd: S,
    pub(crate) buffer: T,
    pub(crate) flags: RecvFlags,
}

impl<T: IoBufMut, S> Recv<T, S> {
    pub fn new(fd: S, buffer: T, flags: RecvFlags) -> Self {
        Self { fd, buffer, flags }
    }
}

impl<T: IoBufMut, S> IntoInner for Recv<T, S> {
    type Inner = T;

    fn into_inner(self) -> Self::Inner {
        self.buffer
    }
}

pub struct Send<T: IoBuf, S> {
    pub(crate) fd: S,
    pub(crate) buffer: T,
    pub(crate) flags: SendFlags,
}

impl<T: IoBuf, S> Send<T, S> {
    pub fn new(fd: S, buffer: T, flags: SendFlags) -> Self {
        Self { fd, buffer, flags }
    }
}

impl<T: IoBuf, S> IntoInner for Send<T, S> {
    type Inner = T;

    fn into_inner(self) -> Self::Inner {
        self.buffer
    }
}

pub struct RecvVectored<T: IoVectoredBufMut, S> {
    pub(crate) fd: S,
    pub(crate) buffer: T,
    pub(crate) flags: RecvFlags,
}

impl<T: IoVectoredBufMut, S> RecvVectored<T, S> {
    pub fn new(fd: S, buffer: T, flags: RecvFlags) -> Self {
        Self { fd, buffer, flags }
    }
}

impl<T: IoVectoredBufMut, S> IntoInner for RecvVectored<T, S> {
    type Inner = T;

    fn into_inner(self) -> Self::Inner {
        self.buffer
    }
}

pub struct SendVectored<T: IoVectoredBuf, S> {
    pub(crate) fd: S,
    pub(crate) buffer: T,
    pub(crate) flags: SendFlags,
}

impl<T: IoVectoredBuf, S> SendVectored<T, S> {
    pub fn new(fd: S, buffer: T, flags: SendFlags) -> Self {
        Self { fd, buffer, flags }
    }
}

impl<T: IoVectoredBuf, S> IntoInner for SendVectored<T, S> {
    type Inner = T;

    fn into_inner(self) -> Self::Inner {
        self.buffer
    }
}

pub struct RecvVectoredControl {
    pub(crate) msg: MsgHdr,
    pub(crate) slices: Vec<IoVec>,
}

impl Default for RecvVectoredControl {
    fn default() -> Self {
        Self {
            msg: unsafe { std::mem::zeroed() },
            slices: Vec::new(),
        }
    }
}

impl<T: IoVectoredBufMut, S> RecvVectored<T, S> {
    pub(crate) fn init_control(&mut self, ctrl: &mut RecvVectoredControl) {
        ctrl.slices = self
            .buffer
            .iter_spare_slice()
            .map(IoVec::from_uninit)
            .collect();
        ctrl.msg.set_mut_vec(ctrl.slices.as_mut_slice());
    }
}

pub struct SendVectoredControl {
    pub(crate) msg: MsgHdr,
    pub(crate) slices: Vec<IoVec>,
}

impl<T: IoVectoredBuf, S> SendVectored<T, S> {
    pub(crate) fn init_control(&mut self, ctrl: &mut SendVectoredControl) {
        ctrl.slices = self
            .buffer
            .iter_pos_slice()
            .map(IoVec::from_slice)
            .collect();
        ctrl.msg.set_vec(ctrl.slices.as_slice());
    }
}

impl Default for SendVectoredControl {
    fn default() -> Self {
        Self {
            msg: MsgHdr::default(),
            slices: Vec::new(),
        }
    }
}
