use super::super::{IoBuf, IoVectoredBuf};
use crate::os::io::IoVec;
use core::{cell::UnsafeCell, mem::MaybeUninit};

pub struct IoVecArray<E, const N: usize> {
    data: E,
    vec: UnsafeCell<[IoVec; N]>,
}

impl<E, const N: usize> IoVecArray<E, N> {
    pub fn as_ref(&self) -> &E {
        &self.data
    }
    pub fn as_mut(&mut self) -> &mut E {
        &mut self.data
    }

    pub fn into(self) -> E {
        self.data
    }

    fn new(data: E) -> Self {
        Self {
            data,
            vec: UnsafeCell::new(unsafe { MaybeUninit::uninit().assume_init() }),
        }
    }
}

impl<E1: IoBuf, E2: IoBuf> IoVecArray<(E1, E2), 2> {
    pub fn new_arg2(v1: E1, v2: E2) -> Self {
        Self::new((v1, v2))
    }
}

impl<E1: IoBuf, E2: IoBuf, const N: usize> IoVectoredBuf for IoVecArray<(E1, E2), N> {
    fn pos_ref(&self) -> &[IoVec] {
        unsafe {
            let vec = self.vec.get().as_mut_unchecked();
            *vec.get_unchecked_mut(0) = IoVec::from_ref(self.data.0.pos_ref());
            *vec.get_unchecked_mut(1) = IoVec::from_ref(self.data.1.pos_ref());

            vec.as_slice()
        }
    }
}

impl<E1: IoBuf, E2: IoBuf, E3: IoBuf> IoVecArray<(E1, E2, E3), 3> {
    pub fn new_arg3(v1: E1, v2: E2, v3: E3) -> Self {
        Self::new((v1, v2, v3))
    }
}

impl<E1: IoBuf, E2: IoBuf, E3: IoBuf, const N: usize> IoVectoredBuf
    for IoVecArray<(E1, E2, E3), N>
{
    fn pos_ref(&self) -> &[IoVec] {
        unsafe {
            let vec = self.vec.get().as_mut_unchecked();
            *vec.get_unchecked_mut(0) = IoVec::from_ref(self.data.0.pos_ref());
            *vec.get_unchecked_mut(1) = IoVec::from_ref(self.data.1.pos_ref());
            *vec.get_unchecked_mut(2) = IoVec::from_ref(self.data.2.pos_ref());

            vec.as_slice()
        }
    }
}

impl<E1: IoBuf, E2: IoBuf, E3: IoBuf, E4: IoBuf> IoVecArray<(E1, E2, E3, E4), 4> {
    pub fn new_arg4(v1: E1, v2: E2, v3: E3, v4: E4) -> Self {
        Self::new((v1, v2, v3, v4))
    }
}

impl<E1: IoBuf, E2: IoBuf, E3: IoBuf, E4: IoBuf, const N: usize> IoVectoredBuf
    for IoVecArray<(E1, E2, E3, E4), N>
{
    fn pos_ref(&self) -> &[IoVec] {
        unsafe {
            let vec = self.vec.get().as_mut_unchecked();
            *vec.get_unchecked_mut(0) = IoVec::from_ref(self.data.0.pos_ref());
            *vec.get_unchecked_mut(1) = IoVec::from_ref(self.data.1.pos_ref());
            *vec.get_unchecked_mut(2) = IoVec::from_ref(self.data.2.pos_ref());
            *vec.get_unchecked_mut(3) = IoVec::from_ref(self.data.3.pos_ref());

            vec.as_slice()
        }
    }
}

impl<E, const N: usize> IoVecArray<[E; N], N>
where
    E: IoBuf,
{
    pub fn new_array(data: [E; N]) -> Self {
        Self::new(data)
    }
}

impl<E, const N: usize> IoVectoredBuf for IoVecArray<[E; N], N>
where
    E: IoBuf,
{
    fn pos_ref(&self) -> &[IoVec] {
        unsafe {
            let vec = self.vec.get().as_mut_unchecked();

            self.data
                .iter()
                .zip(vec.iter_mut())
                .for_each(|(e, v)| *v = IoVec::from_ref(e.pos_ref()));

            vec.as_slice()
        }
    }
}
