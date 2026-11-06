use super::{IoBuf, IoBufMut};
use alloc::{boxed::Box, rc::Rc, sync::Arc, vec::Vec};
use core::{iter::Iterator, mem::MaybeUninit};

pub trait IoVectoredBuf: 'static {
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]>;
}

impl<B: IoVectoredBuf + ?Sized> IoVectoredBuf for &'static B {
    #[inline]
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        (**self).pos_ref()
    }
}

impl<B: IoVectoredBuf + ?Sized> IoVectoredBuf for &'static mut B {
    #[inline]
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        (**self).pos_ref()
    }
}

impl<B: IoVectoredBuf + ?Sized + 'static> IoVectoredBuf for Box<B> {
    #[inline]
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        (**self).pos_ref()
    }
}

impl<B: IoVectoredBuf + ?Sized + 'static> IoVectoredBuf for Rc<B> {
    #[inline]
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        (**self).pos_ref()
    }
}

impl<B: IoVectoredBuf + ?Sized + 'static> IoVectoredBuf for Arc<B> {
    #[inline]
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        (**self).pos_ref()
    }
}

impl<T: IoBuf> IoVectoredBuf for [T] {
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        self.iter().map(|v| v.pos_ref())
    }
}

impl<T: IoBuf, const N: usize> IoVectoredBuf for [T; N] {
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        self.iter().map(|v| v.pos_ref())
    }
}

impl<T: IoBuf> IoVectoredBuf for Vec<T> {
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        self.iter().map(|v| v.pos_ref())
    }
}

impl<T1: IoBuf, T2: IoBuf> IoVectoredBuf for (T1, T2) {
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        [self.0.pos_ref(), self.1.pos_ref()].into_iter()
    }
}

impl<T1: IoBuf, T2: IoBuf, T3: IoBuf> IoVectoredBuf for (T1, T2, T3) {
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        [self.0.pos_ref(), self.1.pos_ref(), self.2.pos_ref()].into_iter()
    }
}

impl<T1: IoBuf, T2: IoBuf, T3: IoBuf, T4: IoBuf> IoVectoredBuf for (T1, T2, T3, T4) {
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        [
            self.0.pos_ref(),
            self.1.pos_ref(),
            self.2.pos_ref(),
            self.3.pos_ref(),
        ]
        .into_iter()
    }
}

pub trait IoVectoredBufMut: 'static {
    fn spare_mut(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]>;
}

impl<B: IoVectoredBufMut + ?Sized> IoVectoredBufMut for &'static mut B {
    #[inline]
    fn spare_mut(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        (**self).spare_mut()
    }
}

impl<B: IoVectoredBufMut + ?Sized + 'static> IoVectoredBufMut for Box<B> {
    #[inline]
    fn spare_mut(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        (**self).spare_mut()
    }
}

impl<T: IoBufMut> IoVectoredBufMut for &'static mut [T] {
    fn spare_mut(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        self.iter_mut().map(|v| v.spare_mut())
    }
}

impl<T: IoBufMut, const N: usize> IoVectoredBufMut for [T; N] {
    fn spare_mut(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        self.iter_mut().map(|v| v.spare_mut())
    }
}

impl<T: IoBufMut> IoVectoredBufMut for Vec<T> {
    fn spare_mut(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        self.iter_mut().map(|v| v.spare_mut())
    }
}

impl<T1: IoBufMut, T2: IoBufMut> IoVectoredBufMut for (T1, T2) {
    fn spare_mut(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        [self.0.spare_mut(), self.1.spare_mut()].into_iter()
    }
}

impl<T1: IoBufMut, T2: IoBufMut, T3: IoBufMut> IoVectoredBufMut for (T1, T2, T3) {
    fn spare_mut(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        [self.0.spare_mut(), self.1.spare_mut(), self.2.spare_mut()].into_iter()
    }
}

impl<T1: IoBufMut, T2: IoBufMut, T3: IoBufMut, T4: IoBufMut> IoVectoredBufMut for (T1, T2, T3, T4) {
    fn spare_mut(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        [
            self.0.spare_mut(),
            self.1.spare_mut(),
            self.2.spare_mut(),
            self.3.spare_mut(),
        ]
        .into_iter()
    }
}
