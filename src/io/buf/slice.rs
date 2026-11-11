use crate::io::{IoBuf, IoBufMut};
use core::{
    mem::MaybeUninit,
    ops::{Bound, Range, RangeBounds},
};

pub struct Slice<T> {
    buffer: T,
    begin: usize,
    end: Option<usize>,
}

impl<T> Slice<T> {
    pub fn new<R>(buffer: T, range: R) -> Slice<T>
    where
        R: RangeBounds<usize>,
    {
        let begin = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n + 1,
            Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            Bound::Included(&n) => Some(n.checked_add(1).expect("out of range")),
            Bound::Excluded(&n) => Some(n),
            Bound::Unbounded => None,
        };

        if let Some(end) = end {
            assert!(begin <= end);
        }
        Self { buffer, begin, end }
    }

    #[inline]
    pub fn begin(&self) -> usize {
        self.begin
    }

    #[inline]
    pub unsafe fn set_begin_unchecked(&mut self, begin: usize) {
        self.begin = begin;
    }

    #[inline]
    pub fn end(&self) -> Option<usize> {
        self.end
    }

    #[inline]
    pub fn set_end(&mut self, end: usize) {
        self.end = Some(end);
    }

    #[inline]
    pub fn as_inner(&self) -> &T {
        &self.buffer
    }

    #[inline]
    pub fn as_inner_mut(&mut self) -> &mut T {
        &mut self.buffer
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.buffer
    }

    unsafe fn new_unchecked(buffer: T, begin: usize, end: Option<usize>) -> Self {
        Self { buffer, begin, end }
    }
}

impl<T: IoBuf> Slice<Slice<T>> {
    pub fn flatten(self) -> Slice<T> {
        let large_begin = self.buffer.begin;
        let large_end = self.buffer.end;

        let new_begin = large_begin + self.begin;
        let new_end = match (self.end, large_end) {
            (Some(small_end), Some(large_end)) => Some((large_begin + small_end).min(large_end)),
            (Some(small_end), None) => Some(large_begin + small_end),
            (None, large_end) => large_end,
        };
        unsafe { Slice::new_unchecked(self.buffer.buffer, new_begin, new_end) }
    }
}

impl<T: IoBuf> Slice<T> {
    #[inline]
    fn pos_end_or_len(&self) -> usize {
        let len = self.buffer.pos_len();
        match self.end {
            Some(end) => len.min(end),
            None => len,
        }
    }

    #[inline]
    fn pos_range(&self) -> Range<usize> {
        let end = self.pos_end_or_len();
        self.begin..end
    }
}

impl<T: IoBufMut> Slice<T> {
    fn spare_end_or_len(&mut self) -> usize {
        let len = self.buffer.spare_len();
        match self.end {
            Some(end) => len.min(end),
            None => len,
        }
    }

    #[inline]
    fn spare_range(&mut self) -> Range<usize> {
        let end = self.spare_end_or_len();
        self.begin..end
    }
}

impl<T: IoBuf> IoBuf for Slice<T> {
    #[inline]
    fn pos_ref(&self) -> &[u8] {
        &self.buffer.pos_ref()[self.pos_range()]
    }

    #[inline]
    fn pos_len(&self) -> usize {
        self.pos_end_or_len() - self.begin
    }

    #[inline]
    fn pos_is_empty(&self) -> bool {
        self.begin == self.pos_end_or_len()
    }
}

impl<T: IoBufMut> IoBufMut for Slice<T> {
    #[inline]
    fn spare_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        let range = self.spare_range();
        &mut self.buffer.spare_mut()[range]
    }

    #[inline]
    fn spare_len(&mut self) -> usize {
        self.spare_end_or_len() - self.begin
    }

    #[inline]
    fn spare_is_empty(&mut self) -> bool {
        self.begin == self.spare_end_or_len()
    }
}
