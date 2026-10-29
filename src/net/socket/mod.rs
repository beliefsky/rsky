use std::{io, os::fd::OwnedFd};

use crate::{
    driver::{
        SharedFd,
        op::{
            Accept, Bind, CloseSocket, Connect, CreateSocket, Listen, Recv, RecvVectored, Send,
            SendVectored, ShutdownSocket,
        },
    },
    io::{BufResult, IntoInner, IoBuf, IoBufMut, IoVectoredBuf, IoVectoredBufMut},
    os::net::{
        AddressFamily, Protocol, RecvFlags, SendFlags, Shutdown, SockAddrArgs, SockAddrArgsMut,
        SocketType,
    },
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

    pub async fn bind<A: SockAddrArgs>(&self, addr: A) -> io::Result<()> {
        runtime::submit(Bind::new(self.shared_fd(), addr))
            .await
            .into_res()
            .map(|_| ())
    }

    pub async fn listen(&self, backlog: i32) -> io::Result<()> {
        runtime::submit(Listen::new(self.shared_fd(), backlog))
            .await
            .into_res()
            .map(|_| ())
    }

    pub async fn connect<A: SockAddrArgs>(&self, addr: A) -> io::Result<()> {
        runtime::submit(Connect::new(self.shared_fd(), addr))
            .await
            .into_res()
            .map(|_| ())
    }

    pub async fn accept<A: SockAddrArgsMut>(&self, addr: A) -> io::Result<(Self, A)> {
        let (res, op) = runtime::submit(Accept::new(self.shared_fd(), addr))
            .await
            .into();
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
                runtime::submit(CloseSocket::new(fd)).await.into_res()?;
            }
            Ok(())
        }
    }

    pub async fn recv<B: IoBufMut>(&self, buffer: B, flags: RecvFlags) -> BufResult<usize, B> {
        runtime::submit(Recv::new(self.shared_fd(), buffer, flags))
            .await
            .into_inner()
    }

    pub async fn send<B: IoBuf>(&self, buffer: B, flags: SendFlags) -> BufResult<usize, B> {
        runtime::submit(Send::new(self.shared_fd(), buffer, flags))
            .await
            .into_inner()
    }

    pub async fn recv_vectored<V: IoVectoredBufMut>(
        &self,
        buffer: V,
        flags: RecvFlags,
    ) -> BufResult<usize, V> {
        runtime::submit(RecvVectored::new(self.shared_fd(), buffer, flags))
            .await
            .into_inner()
    }

    pub async fn send_vectored<V: IoVectoredBuf>(
        &self,
        buffer: V,
        flags: SendFlags,
    ) -> BufResult<usize, V> {
        runtime::submit(SendVectored::new(self.shared_fd(), buffer, flags))
            .await
            .into_inner()
    }

    fn shared_fd(&self) -> SharedFd<OwnedFd> {
        self.socket.shared_fd()
    }
}
