use core::mem::ManuallyDrop;
use std::os::fd::{AsFd, OwnedFd};

use crate::{
    io::{IntoInner, IoBuf, IoBufMut, IoVectoredBuf, IoVectoredBufMut},
    os::net::{
        AddressFamily, IoVec, MsgHdr, Protocol, RecvFlags, SendFlags, Shutdown, SockAddrArgs,
        SockAddrArgsMut, SocketType,
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

pub struct Bind<F, A>
where
    F: AsFd,
    A: SockAddrArgs,
{
    pub(crate) fd: F,
    pub(crate) addr: A,
}

impl<F, A> Bind<F, A>
where
    F: AsFd,
    A: SockAddrArgs,
{
    pub fn new(fd: F, addr: A) -> Self {
        Self { fd, addr }
    }
}

pub struct Listen<F>
where
    F: AsFd,
{
    pub(crate) fd: F,
    pub(crate) backlog: i32,
}

impl<F> Listen<F>
where
    F: AsFd,
{
    pub fn new(fd: F, backlog: i32) -> Self {
        Self { fd, backlog }
    }
}

pub struct Accept<F, A>
where
    F: AsFd,
    A: SockAddrArgsMut,
{
    pub(crate) fd: F,
    pub(crate) addr: A,
    pub(crate) accepted_fd: Option<OwnedFd>,
}

impl<F, A> Accept<F, A>
where
    F: AsFd,
    A: SockAddrArgsMut,
{
    pub fn new(fd: F, addr: A) -> Self {
        Self {
            fd,
            addr: addr,
            accepted_fd: None,
        }
    }
}

impl<F, A> IntoInner for Accept<F, A>
where
    F: AsFd,
    A: SockAddrArgsMut,
{
    type Inner = (OwnedFd, A);

    fn into_inner(mut self) -> Self::Inner {
        let socket = self.accepted_fd.take().expect("socket not accepted");
        (socket, self.addr)
    }
}

pub struct Connect<F, A>
where
    F: AsFd,
    A: SockAddrArgs,
{
    pub(crate) fd: F,
    pub(crate) addr: A,
}

impl<F, A> Connect<F, A>
where
    F: AsFd,
    A: SockAddrArgs,
{
    pub fn new(fd: F, addr: A) -> Self {
        Self { fd, addr }
    }
}

pub struct ShutdownSocket<F>
where
    F: AsFd,
{
    pub(crate) fd: F,
    pub(crate) how: Shutdown,
}

impl<F> ShutdownSocket<F>
where
    F: AsFd,
{
    pub fn new(fd: F, how: Shutdown) -> Self {
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

pub struct Recv<F, B>
where
    F: AsFd,
    B: IoBufMut,
{
    pub(crate) fd: F,
    pub(crate) buffer: B,
    pub(crate) flags: RecvFlags,
}

impl<F, B> Recv<F, B>
where
    F: AsFd,
    B: IoBufMut,
{
    pub fn new(fd: F, buffer: B, flags: RecvFlags) -> Self {
        Self { fd, buffer, flags }
    }
}

impl<F, B> IntoInner for Recv<F, B>
where
    F: AsFd,
    B: IoBufMut,
{
    type Inner = B;

    fn into_inner(self) -> Self::Inner {
        self.buffer
    }
}

pub struct Send<F, B>
where
    F: AsFd,
    B: IoBuf,
{
    pub(crate) fd: F,
    pub(crate) buffer: B,
    pub(crate) flags: SendFlags,
}

impl<F, B> Send<F, B>
where
    F: AsFd,
    B: IoBuf,
{
    pub fn new(fd: F, buffer: B, flags: SendFlags) -> Self {
        Self { fd, buffer, flags }
    }
}

impl<F, B> IntoInner for Send<F, B>
where
    F: AsFd,
    B: IoBuf,
{
    type Inner = B;

    fn into_inner(self) -> Self::Inner {
        self.buffer
    }
}

pub struct RecvVectored<F, B>
where
    F: AsFd,
    B: IoVectoredBufMut,
{
    pub(crate) fd: F,
    pub(crate) buffer: B,
    pub(crate) flags: RecvFlags,
}

impl<F, B> RecvVectored<F, B>
where
    F: AsFd,
    B: IoVectoredBufMut,
{
    pub fn new(fd: F, buffer: B, flags: RecvFlags) -> Self {
        Self { fd, buffer, flags }
    }

    pub(crate) fn init_control(&mut self, ctrl: &mut RecvVectoredControl) {
        ctrl.slices = self
            .buffer
            .iter_spare_slice()
            .map(IoVec::from_uninit)
            .collect();
        ctrl.msg.set_mut_vec(ctrl.slices.as_mut_slice());
    }
}

impl<F, B> IntoInner for RecvVectored<F, B>
where
    F: AsFd,
    B: IoVectoredBufMut,
{
    type Inner = B;

    fn into_inner(self) -> Self::Inner {
        self.buffer
    }
}

pub struct SendVectored<F, B>
where
    F: AsFd,
    B: IoVectoredBuf,
{
    pub(crate) fd: F,
    pub(crate) buffer: B,
    pub(crate) flags: SendFlags,
}

impl<F, B> SendVectored<F, B>
where
    F: AsFd,
    B: IoVectoredBuf,
{
    pub fn new(fd: F, buffer: B, flags: SendFlags) -> Self {
        Self { fd, buffer, flags }
    }

    pub(crate) fn init_control(&mut self, ctrl: &mut SendVectoredControl) {
        ctrl.slices = self
            .buffer
            .iter_pos_slice()
            .map(IoVec::from_slice)
            .collect();
        ctrl.msg.set_vec(ctrl.slices.as_slice());
    }
}

impl<F, B> IntoInner for SendVectored<F, B>
where
    F: AsFd,
    B: IoVectoredBuf,
{
    type Inner = B;

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

pub struct SendVectoredControl {
    pub(crate) msg: MsgHdr,
    pub(crate) slices: Vec<IoVec>,
}

impl Default for SendVectoredControl {
    fn default() -> Self {
        Self {
            msg: MsgHdr::default(),
            slices: Vec::new(),
        }
    }
}
