use crate::io::{Bytes, BytesMut, Slice};
use alloc::{boxed::Box, rc::Rc, string::String, sync::Arc, vec::Vec};
use core::{mem::MaybeUninit, ops::RangeBounds, slice};

const _: [&dyn IoBuf; 0] = [];

pub trait IoBuf: 'static {
    fn pos_ref(&self) -> &[u8];

    #[inline]
    fn pos_len(&self) -> usize {
        self.pos_ref().len()
    }

    #[inline]
    fn pos_is_empty(&self) -> bool {
        self.pos_len() == 0
    }
}

impl<B: IoBuf + ?Sized> IoBuf for &'static B {
    #[inline]
    fn pos_ref(&self) -> &[u8] {
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

impl<B: IoBuf + ?Sized> IoBuf for &'static mut B {
    #[inline]
    fn pos_ref(&self) -> &[u8] {
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

impl<B: IoBuf + ?Sized + 'static> IoBuf for Box<B> {
    #[inline]
    fn pos_ref(&self) -> &[u8] {
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

impl<B: IoBuf + ?Sized + 'static> IoBuf for Rc<B> {
    #[inline]
    fn pos_ref(&self) -> &[u8] {
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

impl<B: IoBuf + ?Sized + 'static> IoBuf for Arc<B> {
    #[inline]
    fn pos_ref(&self) -> &[u8] {
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

impl IoBuf for [u8] {
    #[inline]
    fn pos_ref(&self) -> &[u8] {
        self
    }
}

impl<const N: usize> IoBuf for [u8; N] {
    #[inline]
    fn pos_ref(&self) -> &[u8] {
        self
    }
    #[inline]
    fn pos_len(&self) -> usize {
        N
    }
}

impl IoBuf for str {
    #[inline]
    fn pos_ref(&self) -> &[u8] {
        self.as_bytes()
    }
    #[inline]
    fn pos_is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl IoBuf for String {
    #[inline]
    fn pos_ref(&self) -> &[u8] {
        self.as_bytes()
    }
    #[inline]
    fn pos_len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn pos_is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl IoBuf for Vec<u8> {
    #[inline]
    fn pos_ref(&self) -> &[u8] {
        self
    }
    #[inline]
    fn pos_len(&self) -> usize {
        self.len()
    }
}

impl IoBuf for Bytes {
    #[inline]
    fn pos_ref(&self) -> &[u8] {
        self
    }
    #[inline]
    fn pos_len(&self) -> usize {
        self.len()
    }
}

impl IoBuf for BytesMut {
    #[inline]
    fn pos_ref(&self) -> &[u8] {
        self
    }
    #[inline]
    fn pos_len(&self) -> usize {
        self.len()
    }
}

const _: [&dyn IoBufMut; 0] = [];

pub trait IoBufMut: 'static {
    fn spare_mut(&mut self) -> &mut [MaybeUninit<u8>];

    #[inline]
    fn spare_len(&mut self) -> usize {
        self.spare_mut().len()
    }

    #[inline]
    fn spare_is_empty(&mut self) -> bool {
        self.spare_len() == 0
    }
}

impl<B: IoBufMut + ?Sized> IoBufMut for &'static mut B {
    #[inline]
    fn spare_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        (**self).spare_mut()
    }
    #[inline]
    fn spare_len(&mut self) -> usize {
        (**self).spare_mut().len()
    }

    #[inline]
    fn spare_is_empty(&mut self) -> bool {
        (**self).spare_len() == 0
    }
}

impl<B: IoBufMut + ?Sized + 'static> IoBufMut for Box<B> {
    #[inline]
    fn spare_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        (**self).spare_mut()
    }
    #[inline]
    fn spare_len(&mut self) -> usize {
        (**self).spare_mut().len()
    }

    #[inline]
    fn spare_is_empty(&mut self) -> bool {
        (**self).spare_len() == 0
    }
}

impl IoBufMut for [u8] {
    #[inline]
    fn spare_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        let ptr = self.as_mut_ptr() as *mut MaybeUninit<u8>;
        let len = self.len();
        unsafe { slice::from_raw_parts_mut(ptr, len) }
    }
    #[inline]
    fn spare_len(&mut self) -> usize {
        self.len()
    }
}

impl<const N: usize> IoBufMut for [u8; N] {
    #[inline]
    fn spare_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        let ptr = self.as_mut_ptr() as *mut MaybeUninit<u8>;
        unsafe { slice::from_raw_parts_mut(ptr, N) }
    }
    #[inline]
    fn spare_len(&mut self) -> usize {
        N
    }
}

impl IoBufMut for [MaybeUninit<u8>] {
    #[inline]
    fn spare_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self
    }
    #[inline]
    fn spare_len(&mut self) -> usize {
        self.len()
    }
}

impl<const N: usize> IoBufMut for [MaybeUninit<u8>; N] {
    #[inline]
    fn spare_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self
    }
    #[inline]
    fn spare_len(&mut self) -> usize {
        N
    }
}

impl IoBufMut for Vec<u8> {
    #[inline]
    fn spare_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.spare_capacity_mut()
    }
    #[inline]
    fn spare_len(&mut self) -> usize {
        self.capacity() - self.len()
    }

    #[inline]
    fn spare_is_empty(&mut self) -> bool {
        self.capacity() == self.len()
    }
}

impl IoBufMut for super::super::BytesMut {
    #[inline]
    fn spare_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.spare_capacity_mut()
    }

    #[inline]
    fn spare_len(&mut self) -> usize {
        self.capacity() - self.len()
    }

    #[inline]
    fn spare_is_empty(&mut self) -> bool {
        self.capacity() == self.len()
    }
}

// ================= Ex ===================

pub trait IoBufExt: IoBuf {
    fn slice<R>(self, range: R) -> Slice<Self>
    where
        R: RangeBounds<usize>,
        Self: Sized,
    {
        let len = self.pos_len();
        let slice = unsafe { Slice::new(self, range) };
        assert!(slice.begin() <= len);

        slice
    }
}

pub trait IoBufMutExt: IoBufMut {
    fn slice<R>(mut self, range: R) -> Slice<Self>
    where
        R: RangeBounds<usize>,
        Self: Sized,
    {
        let len = self.spare_len();
        let slice = unsafe { Slice::new(self, range) };
        assert!(slice.begin() <= len);

        slice
    }
}
