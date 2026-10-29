use std::{
    io,
    os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd},
};

use io_uring::{opcode, types};

use super::{
    Accept, Bind, CloseSocket, Connect, CreateSocket, Listen, Recv, RecvVectored,
    RecvVectoredControl, Send, SendVectored, SendVectoredControl, ShutdownSocket,
};
use crate::{
    driver::{Extra, OpCode, OpEntry},
    io::{IoBuf, IoBufMut, IoVectoredBuf, IoVectoredBufMut},
    os::net::{self, SockAddrArgs, SockAddrArgsMut},
};

unsafe impl OpCode for CreateSocket {
    type Control = ();

    fn create_entry(&mut self, _: &mut Self::Control) -> OpEntry {
        opcode::Socket::new(
            self.domain.as_raw() as _,
            self.socket_type.as_raw() as i32 | net::SOCK_CLOEXEC,
            self.protocol.map(|p| p.as_raw().get()).unwrap_or_default() as _,
        )
        .build()
        .into()
    }

    unsafe fn set_result(&mut self, _: &mut Self::Control, res: &io::Result<usize>, _: &Extra) {
        if let Ok(fd) = res {
            let fd = unsafe { OwnedFd::from_raw_fd(*fd as _) };
            self.opened_fd = Some(fd);
        }
    }
}

unsafe impl<F, A> OpCode for Bind<F, A>
where
    F: AsFd,
    A: SockAddrArgs,
{
    type Control = ();

    fn create_entry(&mut self, _: &mut Self::Control) -> OpEntry {
        let addr = self.addr.addr_ref();
        opcode::Bind::new(
            types::Fd(self.fd.as_fd().as_raw_fd()),
            addr.as_ptr(),
            addr.len(),
        )
        .build()
        .into()
    }
}

unsafe impl<F> OpCode for Listen<F>
where
    F: AsFd,
{
    type Control = ();

    fn create_entry(&mut self, _: &mut Self::Control) -> OpEntry {
        opcode::Listen::new(types::Fd(self.fd.as_fd().as_raw_fd()), self.backlog)
            .build()
            .into()
    }
}

unsafe impl<F, A> OpCode for Accept<F, A>
where
    F: AsFd,
    A: SockAddrArgsMut,
{
    type Control = ();

    fn create_entry(&mut self, _: &mut Self::Control) -> OpEntry {
        let addr = unsafe { self.addr.addr_mut().assume_init_mut() };
        addr.reset_len();
        let (addr_ptr, len_ptr) = addr.as_mut_ptr_len();

        let entry = opcode::Accept::new(types::Fd(self.fd.as_fd().as_raw_fd()), addr_ptr, len_ptr)
            .flags(net::SOCK_CLOEXEC)
            .build();
        entry.into()
    }

    unsafe fn set_result(&mut self, _: &mut Self::Control, res: &io::Result<usize>, _: &Extra) {
        if let Ok(fd) = res {
            let fd = unsafe { OwnedFd::from_raw_fd(*fd as _) };
            self.accepted_fd = Some(fd);
        }
    }
}

unsafe impl<F, A> OpCode for Connect<F, A>
where
    F: AsFd,
    A: SockAddrArgs,
{
    type Control = ();

    fn create_entry(&mut self, _: &mut Self::Control) -> OpEntry {
        let addr = self.addr.addr_ref();
        opcode::Connect::new(
            types::Fd(self.fd.as_fd().as_raw_fd()),
            addr.as_ptr(),
            addr.len(),
        )
        .build()
        .into()
    }
}

unsafe impl<F> OpCode for ShutdownSocket<F>
where
    F: AsFd,
{
    type Control = ();

    fn create_entry(&mut self, _: &mut Self::Control) -> OpEntry {
        opcode::Shutdown::new(types::Fd(self.fd.as_fd().as_raw_fd()), self.how.how())
            .build()
            .into()
    }
}

unsafe impl OpCode for CloseSocket {
    type Control = ();

    fn create_entry(&mut self, _: &mut Self::Control) -> OpEntry {
        opcode::Close::new(types::Fd(self.fd.as_fd().as_raw_fd()))
            .build()
            .into()
    }
}

unsafe impl<F, B> OpCode for Recv<F, B>
where
    F: AsFd,
    B: IoBufMut,
{
    type Control = ();

    fn create_entry(&mut self, _: &mut Self::Control) -> OpEntry {
        let slice = self.buffer.spare_mut();

        opcode::Recv::new(
            types::Fd(self.fd.as_fd().as_raw_fd()),
            slice.as_mut_ptr() as _,
            slice.len().try_into().unwrap_or(u32::MAX),
        )
        .flags(self.flags.flags() as _)
        .build()
        .into()
    }
}

unsafe impl<F, B> OpCode for Send<F, B>
where
    F: AsFd,
    B: IoBuf,
{
    type Control = ();

    fn create_entry(&mut self, _: &mut Self::Control) -> OpEntry {
        let slice = self.buffer.data_pos();
        opcode::Send::new(
            types::Fd(self.fd.as_fd().as_raw_fd()),
            slice.as_ptr(),
            slice.len().try_into().unwrap_or(u32::MAX),
        )
        .flags(self.flags.flags() as _)
        .build()
        .into()
    }
}

unsafe impl<F, B> OpCode for RecvVectored<F, B>
where
    F: AsFd,
    B: IoVectoredBufMut,
{
    type Control = RecvVectoredControl;

    unsafe fn init(&mut self, ctrl: &mut Self::Control) {
        self.init_control(ctrl)
    }

    fn create_entry(&mut self, control: &mut Self::Control) -> OpEntry {
        opcode::RecvMsg::new(
            types::Fd(self.fd.as_fd().as_raw_fd()),
            control.msg.as_mut_ptr(),
        )
        .flags(self.flags.flags())
        .build()
        .into()
    }
}

unsafe impl<F, B> OpCode for SendVectored<F, B>
where
    F: AsFd,
    B: IoVectoredBuf,
{
    type Control = SendVectoredControl;

    unsafe fn init(&mut self, ctrl: &mut Self::Control) {
        self.init_control(ctrl)
    }

    fn create_entry(&mut self, control: &mut Self::Control) -> OpEntry {
        opcode::SendMsg::new(types::Fd(self.fd.as_fd().as_raw_fd()), control.msg.as_ptr())
            .flags(self.flags.flags())
            .build()
            .into()
    }
}
