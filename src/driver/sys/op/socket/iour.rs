use std::{
    io,
    os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd},
};

use io_uring::{opcode, types};

use super::{Accept, Bind, CloseSocket, Connect, CreateSocket, Listen, Recv, Send, ShutdownSocket};
use crate::{
    driver::{Extra, OpCode, OpEntry},
    io::{IoBuf, IoBufMut},
    os::net,
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

unsafe impl<S: AsFd> OpCode for Bind<S> {
    type Control = ();

    fn create_entry(&mut self, _: &mut Self::Control) -> OpEntry {
        opcode::Bind::new(
            types::Fd(self.fd.as_fd().as_raw_fd()),
            self.addr.as_ptr(),
            self.addr.len(),
        )
        .build()
        .into()
    }
}

unsafe impl<S: AsFd> OpCode for Listen<S> {
    type Control = ();

    fn create_entry(&mut self, _: &mut Self::Control) -> OpEntry {
        opcode::Listen::new(types::Fd(self.fd.as_fd().as_raw_fd()), self.backlog)
            .build()
            .into()
    }
}

unsafe impl<S: AsFd> OpCode for Accept<S> {
    type Control = ();

    fn create_entry(&mut self, _: &mut Self::Control) -> OpEntry {
        let (addr_ptr, len_ptr) = self.addr.as_ptr_len_mut();

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

unsafe impl<S: AsFd> OpCode for Connect<S> {
    type Control = ();

    fn create_entry(&mut self, _: &mut Self::Control) -> OpEntry {
        opcode::Connect::new(
            types::Fd(self.fd.as_fd().as_raw_fd()),
            self.addr.as_ptr(),
            self.addr.len(),
        )
        .build()
        .into()
    }
}

unsafe impl<S: AsFd> OpCode for ShutdownSocket<S> {
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

unsafe impl<T: IoBufMut, S: AsFd> OpCode for Recv<T, S> {
    type Control = ();

    fn create_entry(&mut self, _: &mut Self::Control) -> OpEntry {
        let slice = self.buffer.as_uninit();

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

unsafe impl<T: IoBuf, S: AsFd> OpCode for Send<T, S> {
    type Control = ();

    fn create_entry(&mut self, _: &mut Self::Control) -> OpEntry {
        let slice = self.buffer.as_init();
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
