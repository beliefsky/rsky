use super::super::IoBuf;
use crate::os::io::IoVec;
use core::mem::MaybeUninit;

pub struct IoVecArray<E, const N: usize> {
    data: E,
    vec: [IoVec; N],
}

impl<E, const N: usize> IoVecArray<E, N> {
    pub fn as_ref(&self) -> &E {
        &self.data
    }
    pub fn as_mut(&mut self) -> &mut E {
        &mut self.data
    }

    pub fn vec_slice(&self) -> &[IoVec] {
        &self.vec
    }

    pub fn into(self) -> E {
        self.data
    }
}

impl<E1, E2> IoVecArray<(E1, E2), 2>
where
    E1: IoBuf,
    E2: IoBuf,
{
    pub fn new_arg2(v1: E1, v2: E2) -> Self {
        let vec = unsafe {
            [
                IoVec::from_ref(v1.data_pos()),
                IoVec::from_ref(v2.data_pos()),
            ]
        };
        Self {
            data: (v1, v2),
            vec: vec,
        }
    }
}

impl<E1, E2, E3> IoVecArray<(E1, E2, E3), 3>
where
    E1: IoBuf,
    E2: IoBuf,
    E3: IoBuf,
{
    pub fn new_arg3(v1: E1, v2: E2, v3: E3) -> Self {
        let vec = unsafe {
            [
                IoVec::from_ref(v1.data_pos()),
                IoVec::from_ref(v2.data_pos()),
                IoVec::from_ref(v3.data_pos()),
            ]
        };
        Self {
            data: (v1, v2, v3),
            vec: vec,
        }
    }
}

impl<E1, E2, E3, E4> IoVecArray<(E1, E2, E3, E4), 4>
where
    E1: IoBuf,
    E2: IoBuf,
    E3: IoBuf,
    E4: IoBuf,
{
    pub fn new_arg4(v1: E1, v2: E2, v3: E3, v4: E4) -> Self {
        let vec = unsafe {
            [
                IoVec::from_ref(v1.data_pos()),
                IoVec::from_ref(v2.data_pos()),
                IoVec::from_ref(v3.data_pos()),
                IoVec::from_ref(v4.data_pos()),
            ]
        };
        Self {
            data: (v1, v2, v3, v4),
            vec: vec,
        }
    }
}

impl<E, const N: usize> IoVecArray<[E; N], N>
where
    E: IoBuf,
{
    pub fn new_array(v: [E; N]) -> Self {
        let vec = unsafe { MaybeUninit::uninit().assume_init() };
        let mut r = Self { data: v, vec };

        r.data
            .iter()
            .zip(r.vec.iter_mut())
            .for_each(|(v, b)| *b = unsafe { IoVec::from_ref(v.data_pos()) });
        r
    }
}
