use std::io;

use crate::{
    io::{BufResult, IoBuf, IoBufMut},
    net::Socket,
    os::net::{Protocol, RecvFlags, SendFlags, Shutdown, SockAddr, SocketType},
};

#[derive(Clone)]
pub struct TcpListener {
    inner: Socket,
}

impl TcpListener {
    pub async fn bind(addr: SockAddr) -> io::Result<Self> {
        let socket = Socket::new(addr.family(), SocketType::STREAM, Some(Protocol::TCP)).await?;

        if let Err(e) = Self::bind_with_opt(&socket, addr).await {
            let _ = socket.close().await;
            Err(e)
        } else {
            Ok(Self { inner: socket })
        }
    }

    #[inline(always)]
    pub async fn accept(&self) -> io::Result<(TcpStream, SockAddr)> {
        let (socket, addr) = self.inner.accept().await?;
        Ok((TcpStream { inner: socket }, addr))
    }

    #[inline(always)]
    pub fn close(self) -> impl Future<Output = io::Result<()>> {
        self.inner.close()
    }

    async fn bind_with_opt(socket: &Socket, addr: SockAddr) -> io::Result<()> {
        // 待处理配置, set_reuse_address

        socket.bind(addr).await?;
        socket.listen(128).await
    }
}

#[derive(Clone)]
pub struct TcpStream {
    inner: Socket,
}

impl TcpStream {
    pub async fn connect(addr: SockAddr) -> io::Result<Self> {
        let socket = Socket::new(addr.family(), SocketType::STREAM, Some(Protocol::TCP)).await?;

        if let Err(e) = socket.connect(addr).await {
            let _ = socket.close().await;
            Err(e)
        } else {
            Ok(Self { inner: socket })
        }
    }

    #[inline(always)]
    pub async fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.inner.shutdown(how).await
    }

    #[inline(always)]
    pub fn close(self) -> impl Future<Output = io::Result<()>> {
        self.inner.close()
    }

    #[inline(always)]
    pub async fn read<B: IoBufMut>(&self, buffer: B) -> BufResult<usize, B> {
        self.inner.recv(buffer, RecvFlags::EMPTY).await
    }

    #[inline(always)]
    pub async fn write<B: IoBuf>(&self, buffer: B) -> BufResult<usize, B> {
        self.inner.send(buffer, SendFlags::MSG_NOSIGNAL).await
    }
}
