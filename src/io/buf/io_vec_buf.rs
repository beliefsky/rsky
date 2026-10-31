use super::super::IoVecArray;
use crate::os::io::IoVec;
use alloc::{rc::Rc, sync::Arc};

pub trait IoVectoredBuf: 'static {
    fn data_pos(&self) -> &[IoVec];
}

impl<B: IoVectoredBuf + ?Sized> IoVectoredBuf for &'static B {
    #[inline]
    fn data_pos(&self) -> &[IoVec] {
        (**self).data_pos()
    }
}

impl<B: IoVectoredBuf + ?Sized> IoVectoredBuf for &'static mut B {
    #[inline]
    fn data_pos(&self) -> &[IoVec] {
        (**self).data_pos()
    }
}

impl<B: IoVectoredBuf + ?Sized + 'static> IoVectoredBuf for Box<B> {
    #[inline]
    fn data_pos(&self) -> &[IoVec] {
        (**self).data_pos()
    }
}

impl<B: IoVectoredBuf + ?Sized + 'static> IoVectoredBuf for Rc<B> {
    #[inline]
    fn data_pos(&self) -> &[IoVec] {
        (**self).data_pos()
    }
}

impl<B: IoVectoredBuf + ?Sized + 'static> IoVectoredBuf for Arc<B> {
    #[inline]
    fn data_pos(&self) -> &[IoVec] {
        (**self).data_pos()
    }
}

impl<E: 'static, const N: usize> IoVectoredBuf for IoVecArray<E, N> {
    #[inline]
    fn data_pos(&self) -> &[IoVec] {
        self.vec_slice()
    }
}

pub trait IoVectoredBufMut: 'static {
    fn spare_mut(&mut self) -> &mut [IoVec];
}

impl<B: IoVectoredBufMut + ?Sized + 'static> IoVectoredBufMut for Box<B> {
    #[inline]
    fn spare_mut(&mut self) -> &mut [IoVec] {
        (**self).spare_mut()
    }
}
