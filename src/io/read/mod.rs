use super::{BufResult, IoBufMut, IoVectoredBufMut};

pub trait AsyncRead {
    fn read<B: IoBufMut>(&mut self, buf: B) -> impl Future<Output = BufResult<usize, B>>;

    fn read_vectored<B: IoVectoredBufMut>(
        &mut self,
        buf: B,
    ) -> impl Future<Output = BufResult<usize, B>>;
}

impl<A: AsyncRead + ?Sized> AsyncRead for &mut A {
    #[inline]
    fn read<B: IoBufMut>(&mut self, buf: B) -> impl Future<Output = BufResult<usize, B>> {
        (**self).read(buf)
    }

    #[inline]
    fn read_vectored<B: IoVectoredBufMut>(
        &mut self,
        buf: B,
    ) -> impl Future<Output = BufResult<usize, B>> {
        (**self).read_vectored(buf)
    }
}

impl<A: AsyncRead + ?Sized> AsyncRead for Box<A> {
    #[inline]
    fn read<B: IoBufMut>(&mut self, buf: B) -> impl Future<Output = BufResult<usize, B>> {
        (**self).read(buf)
    }

    #[inline]
    fn read_vectored<B: IoVectoredBufMut>(
        &mut self,
        buf: B,
    ) -> impl Future<Output = BufResult<usize, B>> {
        (**self).read_vectored(buf)
    }
}
