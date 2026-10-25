use super::{IoBuf, IoBufMut};
use core::mem::MaybeUninit;

pub trait IoVectoredBuf: 'static {
    fn iter_pos_slice(&self) -> impl Iterator<Item = &[u8]>;

    fn pos_len(&self) -> usize {
        self.iter_pos_slice().map(|buf| buf.len()).sum()
    }
}

impl<T: IoBuf> IoVectoredBuf for &'static [T] {
    fn iter_pos_slice(&self) -> impl Iterator<Item = &[u8]> {
        self.iter().map(|buf| buf.data_pos())
    }
}

impl<T: IoBuf> IoVectoredBuf for &'static mut [T] {
    fn iter_pos_slice(&self) -> impl Iterator<Item = &[u8]> {
        self.iter().map(|buf| buf.data_pos())
    }
}

impl<T: IoBuf, const N: usize> IoVectoredBuf for [T; N] {
    fn iter_pos_slice(&self) -> impl Iterator<Item = &[u8]> {
        self.iter().map(|buf| buf.data_pos())
    }
}

impl<T: IoBuf> IoVectoredBuf for Vec<T> {
    fn iter_pos_slice(&self) -> impl Iterator<Item = &[u8]> {
        self.iter().map(|buf| buf.data_pos())
    }
}

pub trait IoVectoredBufMut: 'static {
    fn iter_spare_slice(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]>;

    fn spare_len(&mut self) -> usize {
        self.iter_spare_slice().map(|buf| buf.len()).sum()
    }
}

impl<T: IoBufMut> IoVectoredBufMut for &'static mut [T] {
    fn iter_spare_slice(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        self.iter_mut().map(|buf| buf.spare_mut())
    }
}

impl<T: IoBufMut, const N: usize> IoVectoredBufMut for [T; N] {
    fn iter_spare_slice(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        self.iter_mut().map(|buf| buf.spare_mut())
    }
}

impl<T: IoBufMut> IoVectoredBufMut for Vec<T> {
    fn iter_spare_slice(&mut self) -> impl Iterator<Item = &mut [MaybeUninit<u8>]> {
        self.iter_mut().map(|buf| buf.spare_mut())
    }
}
