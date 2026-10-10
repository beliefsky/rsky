use std::{io, os::fd::OwnedFd};

use crate::{
    driver::{
        SharedFd,
        op::{
            Accept, Bind, CloseSocket, Connect, CreateSocket, Listen, Recv, Send, ShutdownSocket,
        },
    },
    io::{BufResult, IntoInner, IoBuf, IoBufMut},
    os::net::{AddressFamily, Protocol, RecvFlags, SendFlags, Shutdown, SockAddr, SocketType},
    runtime::{self, Attacher},
};

#[derive(Clone)]
pub struct Socket {
    pub(crate) socket: Attacher<OwnedFd>,
}

impl Socket {
    pub fn from_fd(fd: OwnedFd) -> io::Result<Self> {
        Ok(Self {
            socket: Attacher::new(fd)?,
        })
    }

    pub async fn new(
        domain: AddressFamily,
        ty: SocketType,
        protocol: Option<Protocol>,
    ) -> io::Result<Self> {
        let (res, op) = runtime::submit(CreateSocket::new(domain, ty, protocol))
            .await
            .into();
        res?;
        Self::from_fd(op.into_inner())
    }

    pub async fn bind(&self, addr: SockAddr) -> io::Result<()> {
        let (res, _) = runtime::submit(Bind::new(self.shared_fd(), addr))
            .await
            .into();
        res?;

        Ok(())
    }

    pub async fn listen(&self, backlog: i32) -> io::Result<()> {
        let (res, _) = runtime::submit(Listen::new(self.shared_fd(), backlog))
            .await
            .into();
        res?;

        Ok(())
    }

    pub async fn connect(&self, addr: SockAddr) -> io::Result<()> {
        let (res, _) = runtime::submit(Connect::new(self.shared_fd(), addr))
            .await
            .into();
        res?;

        Ok(())
    }

    pub async fn accept(&self) -> io::Result<(Self, SockAddr)> {
        let (res, op) = runtime::submit(Accept::new(self.shared_fd())).await.into();
        res?;
        let (accept_fd, addr) = op.into_inner();
        Ok((Self::from_fd(accept_fd)?, addr))
    }

    pub async fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        let (res, _) = runtime::submit(ShutdownSocket::new(self.shared_fd(), how))
            .await
            .into();
        match res {
            Err(e) if e.kind() == io::ErrorKind::NotConnected => Ok(()),
            v => v.map(|_| ()),
        }
    }

    pub fn close(self) -> impl Future<Output = io::Result<()>> {
        async move {
            let fd = self.socket.into_inner().take().await;
            if let Some(fd) = fd {
                let (res, _) = runtime::submit(CloseSocket::new(fd)).await.into();
                res?;
            }
            Ok(())
        }
    }

    pub async fn recv<B: IoBufMut>(&self, buffer: B, flags: RecvFlags) -> BufResult<usize, B> {
        let res = runtime::submit(Recv::new(self.shared_fd(), buffer, flags))
            .await
            .into_inner();

        res
    }

    pub async fn send<B: IoBuf>(&self, buffer: B, flags: SendFlags) -> BufResult<usize, B> {
        let res = runtime::submit(Send::new(self.shared_fd(), buffer, flags))
            .await
            .into_inner();

        res
    }

    fn shared_fd(&self) -> SharedFd<OwnedFd> {
        self.socket.shared_fd()
    }
}
