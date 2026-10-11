use std::io;

use crate::{
    io::{AsyncRead, AsyncWrite, BufResult, IoBuf, IoBufMut, IoVectoredBuf, IoVectoredBufMut},
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

    #[inline]
    pub async fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.inner.shutdown(how).await
    }

    #[inline]
    pub fn close(self) -> impl Future<Output = io::Result<()>> {
        self.inner.close()
    }

    #[inline]
    pub async fn recv<B: IoBufMut>(&self, buffer: B, flags: RecvFlags) -> BufResult<usize, B> {
        self.inner.recv(buffer, flags).await
    }

    #[inline]
    pub async fn send<B: IoBuf>(&self, buffer: B, flags: SendFlags) -> BufResult<usize, B> {
        self.inner.send(buffer, flags).await
    }

    #[inline]
    pub async fn recv_vectored<V: IoVectoredBufMut>(
        &self,
        buffer: V,
        flags: RecvFlags,
    ) -> BufResult<usize, V> {
        self.inner.recv_vectored(buffer, flags).await
    }

    #[inline]
    pub async fn send_vectored<V: IoVectoredBuf>(
        &self,
        buffer: V,
        flags: SendFlags,
    ) -> BufResult<usize, V> {
        self.inner.send_vectored(buffer, flags).await
    }

    #[inline]
    pub async fn read<B: IoBufMut>(&self, buffer: B) -> BufResult<usize, B> {
        self.recv(buffer, RecvFlags::EMPTY).await
    }

    #[inline]
    pub async fn write<B: IoBuf>(&self, buffer: B) -> BufResult<usize, B> {
        self.send(buffer, SendFlags::MSG_NOSIGNAL).await
    }

    #[inline]
    pub async fn read_vectored<V: IoVectoredBufMut>(&self, buffer: V) -> BufResult<usize, V> {
        self.recv_vectored(buffer, RecvFlags::EMPTY).await
    }

    #[inline]
    pub async fn write_vectored<V: IoVectoredBuf>(&self, buffer: V) -> BufResult<usize, V> {
        self.send_vectored(buffer, SendFlags::MSG_NOSIGNAL).await
    }
}

impl AsyncRead for TcpStream {
    #[inline]
    fn read<B: IoBufMut>(&mut self, buf: B) -> impl Future<Output = BufResult<usize, B>> {
        (&*self).read(buf)
    }

    fn read_vectored<B: IoVectoredBufMut>(
        &mut self,
        buf: B,
    ) -> impl Future<Output = BufResult<usize, B>> {
        (&*self).read_vectored(buf)
    }
}

impl AsyncRead for &TcpStream {
    #[inline]
    fn read<B: IoBufMut>(&mut self, buf: B) -> impl Future<Output = BufResult<usize, B>> {
        (*self).read(buf)
    }

    fn read_vectored<B: IoVectoredBufMut>(
        &mut self,
        buf: B,
    ) -> impl Future<Output = BufResult<usize, B>> {
        (*self).read_vectored(buf)
    }
}

impl AsyncWrite for TcpStream {
    #[inline]
    fn write<B: IoBuf>(&mut self, buf: B) -> impl Future<Output = BufResult<usize, B>> {
        (&*self).write(buf)
    }

    fn write_vectored<B: IoVectoredBuf>(
        &mut self,
        buf: B,
    ) -> impl Future<Output = BufResult<usize, B>> {
        (&*self).write_vectored(buf)
    }

    #[inline]
    fn flush(&mut self) -> impl Future<Output = io::Result<()>> {
        async { Ok(()) }
    }
}

impl AsyncWrite for &TcpStream {
    #[inline]
    fn write<B: IoBuf>(&mut self, buf: B) -> impl Future<Output = BufResult<usize, B>> {
        (*self).write(buf)
    }

    fn write_vectored<B: IoVectoredBuf>(
        &mut self,
        buf: B,
    ) -> impl Future<Output = BufResult<usize, B>> {
        (*self).write_vectored(buf)
    }

    #[inline]
    fn flush(&mut self) -> impl Future<Output = io::Result<()>> {
        async { Ok(()) }
    }
}
