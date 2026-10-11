use super::{BufResult, IoBuf, IoVectoredBuf};
use std::io;

pub trait AsyncWrite {
    fn write<B: IoBuf>(&mut self, buf: B) -> impl Future<Output = BufResult<usize, B>>;

    fn write_vectored<B: IoVectoredBuf>(
        &mut self,
        buf: B,
    ) -> impl Future<Output = BufResult<usize, B>>;

    fn flush(&mut self) -> impl Future<Output = io::Result<()>>;
}

impl<A: AsyncWrite + ?Sized> AsyncWrite for &mut A {
    #[inline]
    fn write<B: IoBuf>(&mut self, buf: B) -> impl Future<Output = BufResult<usize, B>> {
        (**self).write(buf)
    }

    #[inline]
    fn write_vectored<B: IoVectoredBuf>(
        &mut self,
        buf: B,
    ) -> impl Future<Output = BufResult<usize, B>> {
        (**self).write_vectored(buf)
    }

    #[inline]
    fn flush(&mut self) -> impl Future<Output = io::Result<()>> {
        (**self).flush()
    }
}

impl<A: AsyncWrite + ?Sized> AsyncWrite for Box<A> {
    #[inline]
    fn write<B: IoBuf>(&mut self, buf: B) -> impl Future<Output = BufResult<usize, B>> {
        (**self).write(buf)
    }

    #[inline]
    fn write_vectored<B: IoVectoredBuf>(
        &mut self,
        buf: B,
    ) -> impl Future<Output = BufResult<usize, B>> {
        (**self).write_vectored(buf)
    }

    #[inline]
    fn flush(&mut self) -> impl Future<Output = io::Result<()>> {
        (**self).flush()
    }
}
