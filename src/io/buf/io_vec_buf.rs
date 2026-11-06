use crate::os::io::IoVec;
use alloc::{boxed::Box, rc::Rc, sync::Arc};

pub trait IoVectoredBuf: 'static {
    fn pos_ref(&self) -> &[IoVec];
}

impl<B: IoVectoredBuf + ?Sized> IoVectoredBuf for &'static B {
    #[inline]
    fn pos_ref(&self) -> &[IoVec] {
        (**self).pos_ref()
    }
}

impl<B: IoVectoredBuf + ?Sized> IoVectoredBuf for &'static mut B {
    #[inline]
    fn pos_ref(&self) -> &[IoVec] {
        (**self).pos_ref()
    }
}

impl<B: IoVectoredBuf + ?Sized + 'static> IoVectoredBuf for Box<B> {
    #[inline]
    fn pos_ref(&self) -> &[IoVec] {
        (**self).pos_ref()
    }
}

impl<B: IoVectoredBuf + ?Sized + 'static> IoVectoredBuf for Rc<B> {
    #[inline]
    fn pos_ref(&self) -> &[IoVec] {
        (**self).pos_ref()
    }
}

impl<B: IoVectoredBuf + ?Sized + 'static> IoVectoredBuf for Arc<B> {
    #[inline]
    fn pos_ref(&self) -> &[IoVec] {
        (**self).pos_ref()
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
