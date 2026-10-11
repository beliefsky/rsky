use std::{mem::MaybeUninit, rc::Rc, sync::Arc};

const _: [&dyn IoBuf; 0] = [];

pub trait IoBuf: 'static {
    fn as_init(&self) -> &[u8];
}

impl<B: IoBuf + ?Sized> IoBuf for &'static B {
    fn as_init(&self) -> &[u8] {
        (**self).as_init()
    }
}

impl<B: IoBuf + ?Sized> IoBuf for &'static mut B {
    fn as_init(&self) -> &[u8] {
        (**self).as_init()
    }
}

impl<B: IoBuf + ?Sized> IoBuf for Box<B> {
    fn as_init(&self) -> &[u8] {
        (**self).as_init()
    }
}

impl<B: IoBuf + ?Sized> IoBuf for Rc<B> {
    fn as_init(&self) -> &[u8] {
        (**self).as_init()
    }
}

impl<B: IoBuf + ?Sized> IoBuf for Arc<B> {
    fn as_init(&self) -> &[u8] {
        (**self).as_init()
    }
}

impl IoBuf for [u8] {
    fn as_init(&self) -> &[u8] {
        self
    }
}

impl<const N: usize> IoBuf for [u8; N] {
    fn as_init(&self) -> &[u8] {
        self
    }
}

impl IoBuf for str {
    fn as_init(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl IoBuf for String {
    fn as_init(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl IoBuf for Vec<u8> {
    fn as_init(&self) -> &[u8] {
        self
    }
}

const _: [&dyn IoBufMut; 0] = [];

pub trait IoBufMut: IoBuf {
    fn as_uninit(&mut self) -> &mut [MaybeUninit<u8>];
}

impl<B: IoBufMut + ?Sized> IoBufMut for &'static mut B {
    fn as_uninit(&mut self) -> &mut [MaybeUninit<u8>] {
        (**self).as_uninit()
    }
}

impl<B: IoBufMut + ?Sized> IoBufMut for Box<B> {
    fn as_uninit(&mut self) -> &mut [MaybeUninit<u8>] {
        (**self).as_uninit()
    }
}

impl IoBufMut for [u8] {
    fn as_uninit(&mut self) -> &mut [MaybeUninit<u8>] {
        let ptr = self.as_mut_ptr() as *mut MaybeUninit<u8>;
        let len = self.len();
        unsafe { std::slice::from_raw_parts_mut(ptr, len) }
    }
}

impl<const N: usize> IoBufMut for [u8; N] {
    fn as_uninit(&mut self) -> &mut [MaybeUninit<u8>] {
        let ptr = self.as_mut_ptr() as *mut MaybeUninit<u8>;
        unsafe { std::slice::from_raw_parts_mut(ptr, N) }
    }
}

impl IoBufMut for Vec<u8> {
    fn as_uninit(&mut self) -> &mut [MaybeUninit<u8>] {
        let ptr = self.as_mut_ptr() as *mut MaybeUninit<u8>;
        let cap = self.capacity();
        unsafe { std::slice::from_raw_parts_mut(ptr, cap) }
    }
}
