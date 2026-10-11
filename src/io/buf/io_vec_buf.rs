use super::{IoBuf, IoBufMut};
use std::mem::MaybeUninit;

pub trait IoVectoredBuf: 'static {
    fn iter_slice(&self) -> impl Iterator<Item = &[u8]>;

    fn total_len(&self) -> usize {
        self.iter_slice().map(|buf| buf.len()).sum()
    }
}

impl<T: IoBuf> IoVectoredBuf for &'static [T] {
    fn iter_slice(&self) -> impl Iterator<Item = &[u8]> {
        self.iter().map(|buf| buf.as_init())
    }
}

impl<T: IoBuf> IoVectoredBuf for &'static mut [T] {
    fn iter_slice(&self) -> impl Iterator<Item = &[u8]> {
        self.iter().map(|buf| buf.as_init())
    }
}

impl<T: IoBuf, const N: usize> IoVectoredBuf for [T; N] {
    fn iter_slice(&self) -> impl Iterator<Item = &[u8]> {
        self.iter().map(|buf| buf.as_init())
    }
}

impl<T: IoBuf> IoVectoredBuf for Vec<T> {
    fn iter_slice(&self) -> impl Iterator<Item = &[u8]> {
        self.iter().map(|buf| buf.as_init())
    }
}

pub trait IoVectoredBufMut: IoVectoredBuf {
    fn iter_uninit_slice(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]>;

    fn total_capacity(&mut self) -> usize {
        self.iter_uninit_slice().map(|buf| buf.len()).sum()
    }
}

impl<T: IoBufMut> IoVectoredBufMut for &'static mut [T] {
    fn iter_uninit_slice(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        self.iter_mut().map(|buf| buf.as_uninit())
    }
}

impl<T: IoBufMut, const N: usize> IoVectoredBufMut for [T; N] {
    fn iter_uninit_slice(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        self.iter_mut().map(|buf| buf.as_uninit())
    }
}

impl<T: IoBufMut> IoVectoredBufMut for Vec<T> {
    fn iter_uninit_slice(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        self.iter_mut().map(|buf| buf.as_uninit())
    }
}
