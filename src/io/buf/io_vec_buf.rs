use super::{IoBuf, IoBufMut};
use alloc::{boxed::Box, rc::Rc, sync::Arc, vec::Vec};
use core::{iter::Iterator, mem::MaybeUninit};

pub trait IoVectoredBuf: 'static {
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]>;

    fn pos_len(&self) -> usize {
        self.pos_ref().map(|b| b.len()).sum()
    }

    fn pos_is_empty(&self) -> bool {
        self.pos_ref().all(|b| b.is_empty())
    }
}

impl<B: IoVectoredBuf + ?Sized> IoVectoredBuf for &'static B {
    #[inline]
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        (**self).pos_ref()
    }

    #[inline]
    fn pos_len(&self) -> usize {
        (**self).pos_len()
    }

    #[inline]
    fn pos_is_empty(&self) -> bool {
        (**self).pos_is_empty()
    }
}

impl<B: IoVectoredBuf + ?Sized> IoVectoredBuf for &'static mut B {
    #[inline]
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        (**self).pos_ref()
    }

    #[inline]
    fn pos_len(&self) -> usize {
        (**self).pos_len()
    }

    #[inline]
    fn pos_is_empty(&self) -> bool {
        (**self).pos_is_empty()
    }
}

impl<B: IoVectoredBuf + ?Sized + 'static> IoVectoredBuf for Box<B> {
    #[inline]
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        (**self).pos_ref()
    }

    #[inline]
    fn pos_len(&self) -> usize {
        (**self).pos_len()
    }

    #[inline]
    fn pos_is_empty(&self) -> bool {
        (**self).pos_is_empty()
    }
}

impl<B: IoVectoredBuf + ?Sized + 'static> IoVectoredBuf for Rc<B> {
    #[inline]
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        (**self).pos_ref()
    }

    #[inline]
    fn pos_len(&self) -> usize {
        (**self).pos_len()
    }

    #[inline]
    fn pos_is_empty(&self) -> bool {
        (**self).pos_is_empty()
    }
}

impl<B: IoVectoredBuf + ?Sized + 'static> IoVectoredBuf for Arc<B> {
    #[inline]
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        (**self).pos_ref()
    }

    #[inline]
    fn pos_len(&self) -> usize {
        (**self).pos_len()
    }

    #[inline]
    fn pos_is_empty(&self) -> bool {
        (**self).pos_is_empty()
    }
}

impl<T: IoBuf> IoVectoredBuf for [T] {
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        self.iter().map(|v| v.pos_ref())
    }

    fn pos_len(&self) -> usize {
        self.iter().map(|v| v.pos_len()).sum()
    }

    fn pos_is_empty(&self) -> bool {
        self.iter().all(|v| v.pos_is_empty())
    }
}

impl<T: IoBuf, const N: usize> IoVectoredBuf for [T; N] {
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        self.iter().map(|v| v.pos_ref())
    }
    fn pos_len(&self) -> usize {
        self.iter().map(|v| v.pos_len()).sum()
    }

    fn pos_is_empty(&self) -> bool {
        self.iter().all(|v| v.pos_is_empty())
    }
}

impl<T: IoBuf> IoVectoredBuf for Vec<T> {
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        self.iter().map(|v| v.pos_ref())
    }
    fn pos_len(&self) -> usize {
        self.iter().map(|v| v.pos_len()).sum()
    }

    fn pos_is_empty(&self) -> bool {
        self.iter().all(|v| v.pos_is_empty())
    }
}

impl<T1: IoBuf, T2: IoBuf> IoVectoredBuf for (T1, T2) {
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        [self.0.pos_ref(), self.1.pos_ref()].into_iter()
    }

    fn pos_len(&self) -> usize {
        self.0.pos_len() + self.1.pos_len()
    }

    fn pos_is_empty(&self) -> bool {
        self.0.pos_is_empty() && self.1.pos_is_empty()
    }
}

impl<T1: IoBuf, T2: IoBuf, T3: IoBuf> IoVectoredBuf for (T1, T2, T3) {
    fn pos_ref(&self) -> impl Iterator<Item = &[u8]> {
        [self.0.pos_ref(), self.1.pos_ref(), self.2.pos_ref()].into_iter()
    }

    fn pos_len(&self) -> usize {
        self.0.pos_len() + self.1.pos_len() + self.2.pos_len()
    }

    fn pos_is_empty(&self) -> bool {
        self.0.pos_is_empty() && self.1.pos_is_empty() && self.2.pos_is_empty()
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

    fn pos_len(&self) -> usize {
        self.0.pos_len() + self.1.pos_len() + self.2.pos_len() + self.3.pos_len()
    }

    fn pos_is_empty(&self) -> bool {
        self.0.pos_is_empty()
            && self.1.pos_is_empty()
            && self.2.pos_is_empty()
            && self.3.pos_is_empty()
    }
}

pub trait IoVectoredBufMut: 'static {
    fn spare_mut(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]>;

    fn spare_len(&mut self) -> usize {
        self.spare_mut().map(|b| b.len()).sum()
    }

    fn spare_is_empty(&mut self) -> bool {
        self.spare_mut().all(|b| b.is_empty())
    }
}

impl<B: IoVectoredBufMut + ?Sized> IoVectoredBufMut for &'static mut B {
    #[inline]
    fn spare_mut(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        (**self).spare_mut()
    }

    #[inline]
    fn spare_len(&mut self) -> usize {
        (**self).spare_len()
    }

    #[inline]
    fn spare_is_empty(&mut self) -> bool {
        (**self).spare_is_empty()
    }
}

impl<B: IoVectoredBufMut + ?Sized + 'static> IoVectoredBufMut for Box<B> {
    #[inline]
    fn spare_mut(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        (**self).spare_mut()
    }

    #[inline]
    fn spare_len(&mut self) -> usize {
        (**self).spare_len()
    }

    #[inline]
    fn spare_is_empty(&mut self) -> bool {
        (**self).spare_is_empty()
    }
}

impl<T: IoBufMut> IoVectoredBufMut for &'static mut [T] {
    fn spare_mut(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        self.iter_mut().map(|v| v.spare_mut())
    }

    fn spare_len(&mut self) -> usize {
        self.iter_mut().map(|v| v.spare_len()).sum()
    }

    fn spare_is_empty(&mut self) -> bool {
        self.iter_mut().all(|v| v.spare_is_empty())
    }
}

impl<T: IoBufMut, const N: usize> IoVectoredBufMut for [T; N] {
    fn spare_mut(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        self.iter_mut().map(|v| v.spare_mut())
    }

    fn spare_len(&mut self) -> usize {
        self.iter_mut().map(|v| v.spare_len()).sum()
    }

    fn spare_is_empty(&mut self) -> bool {
        self.iter_mut().all(|v| v.spare_is_empty())
    }
}

impl<T: IoBufMut> IoVectoredBufMut for Vec<T> {
    fn spare_mut(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        self.iter_mut().map(|v| v.spare_mut())
    }

    fn spare_len(&mut self) -> usize {
        self.iter_mut().map(|v| v.spare_len()).sum()
    }

    fn spare_is_empty(&mut self) -> bool {
        self.iter_mut().all(|v| v.spare_is_empty())
    }
}

impl<T1: IoBufMut, T2: IoBufMut> IoVectoredBufMut for (T1, T2) {
    fn spare_mut(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        [self.0.spare_mut(), self.1.spare_mut()].into_iter()
    }

    fn spare_len(&mut self) -> usize {
        self.0.spare_len() + self.1.spare_len()
    }

    fn spare_is_empty(&mut self) -> bool {
        self.0.spare_is_empty() && self.1.spare_is_empty()
    }
}

impl<T1: IoBufMut, T2: IoBufMut, T3: IoBufMut> IoVectoredBufMut for (T1, T2, T3) {
    fn spare_mut(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        [self.0.spare_mut(), self.1.spare_mut(), self.2.spare_mut()].into_iter()
    }

    fn spare_len(&mut self) -> usize {
        self.0.spare_len() + self.1.spare_len() + self.2.spare_len()
    }

    fn spare_is_empty(&mut self) -> bool {
        self.0.spare_is_empty() && self.1.spare_is_empty() && self.2.spare_is_empty()
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

    fn spare_len(&mut self) -> usize {
        self.0.spare_len() + self.1.spare_len() + self.2.spare_len() + self.3.spare_len()
    }

    fn spare_is_empty(&mut self) -> bool {
        self.0.spare_is_empty()
            && self.1.spare_is_empty()
            && self.2.spare_is_empty()
            && self.3.spare_is_empty()
    }
}
