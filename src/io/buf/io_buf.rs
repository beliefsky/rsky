use alloc::{rc::Rc, sync::Arc};
use core::{mem::MaybeUninit, slice};

const _: [&dyn IoBuf; 0] = [];

pub trait IoBuf: 'static {
    fn data_pos(&self) -> &[u8];
}

impl<B: IoBuf + ?Sized> IoBuf for &'static B {
    #[inline]
    fn data_pos(&self) -> &[u8] {
        (**self).data_pos()
    }
}

impl<B: IoBuf + ?Sized> IoBuf for &'static mut B {
    #[inline]
    fn data_pos(&self) -> &[u8] {
        (**self).data_pos()
    }
}

impl<B: IoBuf + ?Sized + 'static> IoBuf for Box<B> {
    #[inline]
    fn data_pos(&self) -> &[u8] {
        (**self).data_pos()
    }
}

impl<B: IoBuf + ?Sized + 'static> IoBuf for Rc<B> {
    #[inline]
    fn data_pos(&self) -> &[u8] {
        (**self).data_pos()
    }
}

impl<B: IoBuf + ?Sized + 'static> IoBuf for Arc<B> {
    #[inline]
    fn data_pos(&self) -> &[u8] {
        (**self).data_pos()
    }
}

impl IoBuf for [u8] {
    #[inline]
    fn data_pos(&self) -> &[u8] {
        self
    }
}

impl<const N: usize> IoBuf for [u8; N] {
    #[inline]
    fn data_pos(&self) -> &[u8] {
        self
    }
}

impl IoBuf for str {
    #[inline]
    fn data_pos(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl IoBuf for String {
    #[inline]
    fn data_pos(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl IoBuf for Vec<u8> {
    #[inline]
    fn data_pos(&self) -> &[u8] {
        self
    }
}

impl IoBuf for super::super::Bytes {
    #[inline]
    fn data_pos(&self) -> &[u8] {
        self
    }
}

impl IoBuf for super::super::BytesMut {
    #[inline]
    fn data_pos(&self) -> &[u8] {
        self
    }
}

const _: [&dyn IoBufMut; 0] = [];

pub trait IoBufMut: 'static {
    fn spare_mut(&mut self) -> &mut [MaybeUninit<u8>];
}

impl<B: IoBufMut + ?Sized> IoBufMut for &'static mut B {
    #[inline]
    fn spare_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        (**self).spare_mut()
    }
}

impl<B: IoBufMut + ?Sized + 'static> IoBufMut for Box<B> {
    #[inline]
    fn spare_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        (**self).spare_mut()
    }
}

impl IoBufMut for [u8] {
    #[inline]
    fn spare_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        let ptr = self.as_mut_ptr() as *mut MaybeUninit<u8>;
        let len = self.len();
        unsafe { slice::from_raw_parts_mut(ptr, len) }
    }
}

impl<const N: usize> IoBufMut for [u8; N] {
    #[inline]
    fn spare_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        let ptr = self.as_mut_ptr() as *mut MaybeUninit<u8>;
        unsafe { std::slice::from_raw_parts_mut(ptr, N) }
    }
}

impl IoBufMut for [MaybeUninit<u8>] {
    #[inline]
    fn spare_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self
    }
}

impl<const N: usize> IoBufMut for [MaybeUninit<u8>; N] {
    #[inline]
    fn spare_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self
    }
}

impl IoBufMut for Vec<u8> {
    #[inline]
    fn spare_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.spare_capacity_mut()
    }
}

impl IoBufMut for super::super::BytesMut {
    #[inline]
    fn spare_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.spare_capacity_mut()
    }
}
