use crate::{
    io::{
        self, AsyncRead, AsyncWrite, BufResult, IoBuf, IoBufMut, IoVectoredBuf, IoVectoredBufMut,
    },
    net::Socket,
    os::net::{
        Protocol, RecvFlags, SendFlags, Shutdown, SockAddrArgs, SockAddrArgsMut, SocketType,
    },
};

#[derive(Clone)]
#[repr(transparent)]
pub struct TcpListener {
    inner: Socket,
}

impl TcpListener {
    pub async fn bind<Addr: SockAddrArgs>(
        addr: Addr,
        opts: &TcpListenerOptions,
    ) -> io::Result<Self> {
        let socket =
            Socket::new(addr.addr_family(), SocketType::STREAM, Some(Protocol::TCP)).await?;

        if let Err(e) = Self::bind_with_opt(&socket, addr, opts).await {
            let _ = socket.close().await;
            Err(e)
        } else {
            Ok(Self { inner: socket })
        }
    }

    pub fn opts() -> TcpListenerOptions {
        TcpListenerOptions::default()
    }

    #[inline(always)]
    pub async fn accept<A: SockAddrArgsMut>(&self, addr: A) -> io::Result<(TcpStream, A)> {
        let (socket, addr) = self.inner.accept(addr).await?;
        Ok((TcpStream { inner: socket }, addr))
    }

    #[inline(always)]
    pub fn close(self) -> impl Future<Output = io::Result<()>> {
        self.inner.close()
    }

    async fn bind_with_opt<A: SockAddrArgs>(
        socket: &Socket,
        addr: A,
        opts: &TcpListenerOptions,
    ) -> io::Result<()> {
        // 待处理配置, set_reuse_address

        socket.bind(addr).await?;
        socket.listen(opts.backlog).await
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct TcpStream {
    inner: Socket,
}

impl TcpStream {
    pub async fn connect<A: SockAddrArgs>(addr: A) -> io::Result<Self> {
        let socket =
            Socket::new(addr.addr_family(), SocketType::STREAM, Some(Protocol::TCP)).await?;

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

pub struct TcpListenerOptions {
    backlog: i32,
}

impl TcpListenerOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn backlog(&mut self, value: i32) -> &mut Self {
        self.backlog = value;
        self
    }

    pub async fn bind<Addr: SockAddrArgs>(&self, addr: Addr) -> io::Result<TcpListener> {
        TcpListener::bind(addr, self).await
    }
}

impl Default for TcpListenerOptions {
    fn default() -> Self {
        Self { backlog: 128 }
    }
}
