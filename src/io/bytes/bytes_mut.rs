use super::{Bytes, Vtable};
use core::{
    cmp, fmt, hash,
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut, RangeBounds},
    ptr::{self, NonNull},
    slice,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

use alloc::borrow::{Borrow, BorrowMut};

pub struct BytesMut {
    ptr: NonNull<u8>,
    len: usize,
    cap: usize,
    data: *mut Shared,
}

impl BytesMut {
    const KIND_ARC: usize = 0b0;
    const KIND_VEC: usize = 0b1;
    const KIND_MASK: usize = 0b1;

    /// # Examples
    ///
    /// ```
    /// use rsky::io::BytesMut;
    ///
    /// let mut bytes = BytesMut::new();
    ///
    /// assert_eq!(0, bytes.len());
    ///
    /// bytes.reserve(2);
    /// bytes.extend_from_slice(b"xy");
    ///
    /// assert_eq!(&b"xy"[..], &bytes[..]);
    /// ```
    #[inline]
    pub fn new() -> BytesMut {
        BytesMut::with_capacity(0)
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::BytesMut;
    ///
    /// let mut bytes = BytesMut::with_capacity(64);
    ///
    /// // `bytes` contains no data, even though there is capacity
    /// assert_eq!(bytes.len(), 0);
    ///
    /// bytes.extend_from_slice(&b"hello world"[..]);
    ///
    /// assert_eq!(&bytes[..], b"hello world");
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> BytesMut {
        BytesMut::from_vec(Vec::with_capacity(capacity))
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::BytesMut;
    ///
    /// let zeros = BytesMut::zeroed(42);
    ///
    /// assert!(zeros.capacity() >= 42);
    /// assert_eq!(zeros.len(), 42);
    /// zeros.into_iter().for_each(|x| assert_eq!(x, 0));
    /// ```
    pub fn zeroed(len: usize) -> BytesMut {
        BytesMut::from_vec(vec![0; len])
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::BytesMut;
    ///
    /// let b = BytesMut::from(&b"hello"[..]);
    /// assert_eq!(b.len(), 5);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::BytesMut;
    ///
    /// let b = BytesMut::with_capacity(64);
    /// assert!(b.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::BytesMut;
    ///
    /// let b = BytesMut::with_capacity(64);
    /// assert_eq!(b.capacity(), 64);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// # Examples
    ///
    /// ```
    ///
    /// use rsky::io::BytesMut;
    ///
    /// let mut b = BytesMut::from(&b"hello world"[..]);
    ///
    /// unsafe {
    ///     b.set_len(5);
    /// }
    ///
    /// assert_eq!(&b[..], b"hello");
    ///
    /// unsafe {
    ///     b.set_len(11);
    /// }
    ///
    /// assert_eq!(&b[..], b"hello world");
    /// ```
    #[inline]
    pub unsafe fn set_len(&mut self, len: usize) {
        self.len = len;
    }

    /// # Examples
    ///
    /// ```
    ///
    /// use rsky::io::BytesMut;
    ///
    /// let mut buf = BytesMut::from(&b"hello world"[..]);
    /// buf.truncate(5);
    /// assert_eq!(buf, b"hello"[..]);
    /// ```
    pub fn truncate(&mut self, len: usize) {
        if len <= self.len() {
            unsafe { self.set_len(len) };
        }
    }

    /// # Examples
    ///
    /// ```
    ///
    /// use rsky::io::BytesMut;
    ///
    /// let mut buf = BytesMut::from(&b"hello world"[..]);
    /// buf.clear();
    /// assert!(buf.is_empty());
    /// ```
    pub fn clear(&mut self) {
        unsafe { self.set_len(0) };
    }

    /// # Examples
    ///
    /// ```
    ///
    /// use rsky::io::BytesMut;
    ///
    /// let mut buf = BytesMut::from(&b"hello"[..]);
    /// buf.reserve(64);
    /// assert!(buf.capacity() >= 69);
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        let len = self.len();
        let rem = self.capacity() - len;

        if additional > rem {
            let _ = self.reserve_inner(additional, true);
        }
    }

    /// # Examples
    ///
    /// ```
    ///
    /// use rsky::io::BytesMut;
    ///
    /// let mut buf = BytesMut::with_capacity(64);
    /// assert_eq!(true, buf.try_reclaim(64));
    /// assert_eq!(64, buf.capacity());
    ///
    /// buf.extend_from_slice(b"abcd");
    /// let mut split = buf.split();
    /// assert_eq!(60, buf.capacity());
    /// assert_eq!(4, split.capacity());
    /// assert_eq!(false, split.try_reclaim(64));
    /// assert_eq!(false, buf.try_reclaim(64));
    /// // The split buffer is filled with "abcd"
    /// assert_eq!(false, split.try_reclaim(4));
    /// // buf is empty and has capacity for 60 bytes
    /// assert_eq!(true, buf.try_reclaim(60));
    ///
    /// drop(buf);
    /// assert_eq!(false, split.try_reclaim(64));
    ///
    /// split.clear();
    /// assert_eq!(4, split.capacity());
    /// assert_eq!(true, split.try_reclaim(64));
    /// assert_eq!(64, split.capacity());
    /// ```
    #[inline]
    pub fn try_reclaim(&mut self, additional: usize) -> bool {
        let len = self.len();
        let rem = self.capacity() - len;

        if additional <= rem {
            return true;
        }

        self.reserve_inner(additional, false)
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::BytesMut;
    ///
    /// let mut a = BytesMut::from(&b"hello world"[..]);
    /// let mut b = a.split_off(5);
    ///
    /// a[0] = b'j';
    /// b[0] = b'!';
    ///
    /// assert_eq!(&a[..], b"jello");
    /// assert_eq!(&b[..], b"!world");
    /// ```
    pub fn split_off(&mut self, at: usize) -> BytesMut {
        unsafe {
            let mut other = self.shallow_clone();
            other.advance_unchecked(at);
            self.cap = at;
            self.len = cmp::min(self.len, at);
            other
        }
    }

    /// # Examples
    ///
    /// ```
    ///
    /// use rsky::io::BytesMut;
    ///
    /// let mut buf = BytesMut::with_capacity(1024);
    /// buf.extend_from_slice(&b"hello world"[..]);
    ///
    /// let other = buf.split();
    ///
    /// assert!(buf.is_empty());
    /// assert_eq!(1013, buf.capacity());
    ///
    /// assert_eq!(other, b"hello world"[..]);
    /// ```
    pub fn split(&mut self) -> BytesMut {
        let len = self.len();
        self.split_to(len)
    }

    /// # Examples
    ///
    /// ```
    ///
    /// use rsky::io::BytesMut;
    ///
    /// let mut a = BytesMut::from(&b"hello world"[..]);
    /// let mut b = a.split_to(5);
    ///
    /// a[0] = b'!';
    /// b[0] = b'j';
    ///
    /// assert_eq!(&a[..], b"!world");
    /// assert_eq!(&b[..], b"jello");
    /// ```
    pub fn split_to(&mut self, at: usize) -> BytesMut {
        unsafe {
            let mut other = self.shallow_clone();
            self.advance_unchecked(at);
            other.cap = at;
            other.len = at;
            other
        }
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::BytesMut;
    ///
    /// // Allocate buffer big enough for 10 bytes.
    /// let mut buf = BytesMut::with_capacity(10);
    ///
    /// // Fill in the first 3 elements.
    /// let uninit = buf.spare_capacity_mut();
    /// uninit[0].write(0);
    /// uninit[1].write(1);
    /// uninit[2].write(2);
    ///
    /// // Mark the first 3 bytes of the buffer as being initialized.
    /// unsafe {
    ///     buf.set_len(3);
    /// }
    ///
    /// assert_eq!(&buf[..], &[0, 1, 2]);
    /// ```
    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        unsafe {
            let ptr = self.ptr.as_ptr().add(self.len);
            let len = self.cap - self.len;

            slice::from_raw_parts_mut(ptr.cast(), len)
        }
    }

    /// # Examples
    ///
    /// ```
    ///
    /// use rsky::io::BytesMut;
    ///
    /// let mut buf = BytesMut::new();
    ///
    /// buf.resize(3, 0x1);
    /// assert_eq!(&buf[..], &[0x1, 0x1, 0x1]);
    ///
    /// buf.resize(2, 0x2);
    /// assert_eq!(&buf[..], &[0x1, 0x1]);
    ///
    /// buf.resize(4, 0x3);
    /// assert_eq!(&buf[..], &[0x1, 0x1, 0x3, 0x3]);
    /// ```
    pub fn resize(&mut self, new_len: usize, value: u8) {
        let additional = if let Some(additional) = new_len.checked_sub(self.len()) {
            additional
        } else {
            self.truncate(new_len);
            return;
        };
        if additional == 0 {
            return;
        }
        self.reserve(additional);
        let dst = self.spare_capacity_mut().as_mut_ptr();
        unsafe { ptr::write_bytes(dst, value, additional) };
        unsafe { self.set_len(new_len) };
    }

    /// # Examples
    ///
    /// ```
    ///
    /// use rsky::io::BytesMut;
    ///
    /// let mut buf = BytesMut::with_capacity(0);
    /// buf.extend_from_slice(b"aaabbb");
    /// buf.extend_from_slice(b"cccddd");
    ///
    /// assert_eq!(b"aaabbbcccddd", &buf[..]);
    /// ```
    pub fn extend_from_slice(&mut self, extend: &[u8]) {
        let cnt = extend.len();
        self.reserve(cnt);
        let dst = self.spare_capacity_mut();

        unsafe {
            ptr::copy_nonoverlapping(extend.as_ptr(), dst.as_mut_ptr().cast(), cnt);
            self.set_len(self.len + cnt);
        }
    }

    /// # Examples
    ///
    /// ```
    ///
    /// use rsky::io::BytesMut;
    ///
    /// let mut buf = BytesMut::with_capacity(0);
    /// buf.extend_from_slice(b"aaabbb_");
    /// buf.extend_from_within(3..6);
    ///
    /// assert_eq!(b"aaabbb_bbb", &buf[..]);
    /// ```
    pub fn extend_from_within(&mut self, range: impl RangeBounds<usize>) {
        let (begin, end) = super::range(range, self.len());

        let cnt = end - begin;
        self.reserve(cnt);

        let src = unsafe { self.as_ptr().add(begin) };
        let dst = self.spare_capacity_mut();

        unsafe {
            ptr::copy_nonoverlapping(src, dst.as_mut_ptr().cast(), cnt);
            self.set_len(self.len + cnt);
        }
    }

    /// # Examples
    ///
    /// ```
    ///
    /// use rsky::io::BytesMut;
    ///
    /// let mut buf = BytesMut::with_capacity(64);
    /// buf.extend_from_slice(b"aaabbbcccddd");
    ///
    /// let split = buf.split_off(6);
    /// assert_eq!(b"aaabbb", &buf[..]);
    /// assert_eq!(b"cccddd", &split[..]);
    ///
    /// buf.unsplit(split);
    /// assert_eq!(b"aaabbbcccddd", &buf[..]);
    /// ```
    pub fn unsplit(&mut self, other: BytesMut) {
        if self.is_empty() {
            *self = other;
            return;
        }
        if let Err(other) = self.try_unsplit(other) {
            self.extend_from_slice(other.as_ref());
        }
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::BytesMut;
    ///
    /// let mut buf = BytesMut::with_capacity(64);
    /// buf.extend_from_slice(b"aaabbbcccddd");
    ///
    /// let mut split_1 = buf.split_off(3);
    /// let split_2 = split_1.split_off(3);
    /// assert_eq!(b"aaa", &buf[..]);
    /// assert_eq!(b"bbb", &split_1[..]);
    /// assert_eq!(b"cccddd", &split_2[..]);
    ///
    /// let split_2 = buf.try_unsplit(split_2).unwrap_err();
    ///
    /// buf.try_unsplit(split_1).unwrap();
    /// buf.try_unsplit(split_2).unwrap();
    /// assert_eq!(b"aaabbbcccddd", &buf[..]);
    /// ```
    pub fn try_unsplit(&mut self, other: BytesMut) -> Result<(), BytesMut> {
        if other.capacity() == 0 {
            return Ok(());
        }

        let ptr = unsafe { self.ptr.as_ptr().add(self.len) };
        if ptr == other.ptr.as_ptr()
            && self.kind() == Self::KIND_ARC
            && other.kind() == Self::KIND_ARC
            && self.data == other.data
        {
            self.len += other.len;
            self.cap += other.cap;
            Ok(())
        } else {
            Err(other)
        }
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::{BytesMut, Bytes};
    ///
    /// let mut b = BytesMut::with_capacity(64);
    /// b.extend_from_slice(&b"hello world"[..]);
    /// let b1 = b.freeze();
    /// let b2 = b1.clone();
    ///
    /// assert_eq!(&b1[..], b"hello world");
    /// assert_eq!(&b2[..], b"hello world");
    /// ```
    #[inline]
    pub fn freeze(self) -> Bytes {
        let bytes = ManuallyDrop::new(self);
        if bytes.kind() == Self::KIND_VEC {
            // Just re-use `Bytes` internal Vec vtable
            unsafe {
                let off = bytes.get_vec_pos();
                let vec = rebuild_vec(bytes.ptr.as_ptr(), bytes.len, bytes.cap, off);
                let mut b: Bytes = vec.into();
                b.inc_start(off);
                b
            }
        } else {
            let ptr = bytes.ptr.as_ptr();
            let len = bytes.len;
            let data = AtomicPtr::new(bytes.data.cast());
            unsafe { Bytes::with_vtable(ptr, len, data, &Shared::VTABLE) }
        }
    }

    #[inline]
    pub(super) fn from_vec(vec: Vec<u8>) -> BytesMut {
        let mut vec = ManuallyDrop::new(vec);
        let ptr = unsafe { NonNull::new_unchecked(vec.as_mut_ptr()) };
        let len = vec.len();
        let cap = vec.capacity();

        let original_capacity_repr = original_capacity_to_repr(cap);
        let data = (original_capacity_repr << ORIGINAL_CAPACITY_OFFSET) | Self::KIND_VEC;

        BytesMut {
            ptr,
            len,
            cap,
            data: invalid_ptr(data),
        }
    }

    pub(super) unsafe fn advance_unchecked(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        let kind = self.kind();

        if kind == Self::KIND_VEC {
            let pos = unsafe { self.get_vec_pos() + count };

            if pos <= MAX_VEC_POS {
                unsafe {
                    self.set_vec_pos(pos);
                }
            } else {
                unsafe {
                    self.promote_to_shared(/*ref_count = */ 1);
                }
            }
        }
        self.ptr = unsafe { NonNull::new_unchecked(self.ptr.as_ptr().add(count)) };
        self.len = self.len.saturating_sub(count);
        self.cap -= count;
    }

    #[inline]
    fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    #[inline]
    fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }

    #[inline]
    fn kind(&self) -> usize {
        self.data as usize & Self::KIND_MASK
    }

    #[inline]
    unsafe fn get_vec_pos(&self) -> usize {
        self.data as usize >> VEC_POS_OFFSET
    }

    #[inline]
    unsafe fn set_vec_pos(&mut self, pos: usize) {
        self.data = invalid_ptr((pos << VEC_POS_OFFSET) | (self.data as usize & NOT_VEC_POS_MASK));
    }

    #[inline]
    unsafe fn shallow_clone(&mut self) -> BytesMut {
        if self.kind() == Self::KIND_ARC {
            unsafe {
                Shared::increment_shared(self.data);
            }
        } else {
            unsafe {
                self.promote_to_shared(/*ref_count = */ 2);
            }
        }
        unsafe { ptr::read(self) }
    }

    unsafe fn promote_to_shared(&mut self, ref_cnt: usize) {
        let original_capacity_repr =
            (self.data as usize & ORIGINAL_CAPACITY_MASK) >> ORIGINAL_CAPACITY_OFFSET;

        let off = (self.data as usize) >> VEC_POS_OFFSET;
        let shared = Box::new(MaybeUninit::<Shared>::uninit());
        let shared = Shared::init_to_raw(
            shared,
            Shared {
                vec: unsafe { rebuild_vec(self.ptr.as_ptr(), self.len, self.cap, off) },
                original_capacity_repr,
                ref_count: AtomicUsize::new(ref_cnt),
            },
        );
        self.data = shared;
    }

    fn reserve_inner(&mut self, additional: usize, allocate: bool) -> bool {
        let len = self.len();
        if self.kind() == Self::KIND_VEC {
            unsafe {
                let off = self.get_vec_pos();
                if self.capacity() - self.len() + off >= additional && off >= self.len() {
                    let base_ptr = self.ptr.as_ptr().sub(off);
                    ptr::copy_nonoverlapping(self.ptr.as_ptr(), base_ptr, self.len);
                    self.ptr = NonNull::new_unchecked(base_ptr);
                    self.set_vec_pos(0);
                    self.cap += off;
                } else {
                    if !allocate {
                        return false;
                    }
                    let mut v =
                        ManuallyDrop::new(rebuild_vec(self.ptr.as_ptr(), self.len, self.cap, off));
                    v.reserve(additional);

                    self.ptr = NonNull::new_unchecked(v.as_mut_ptr().add(off));
                    self.cap = v.capacity() - off;
                }

                return true;
            }
        }
        let shared: *mut Shared = self.data;

        let mut new_cap = match len.checked_add(additional) {
            Some(new_cap) => new_cap,
            None if !allocate => return false,
            None => panic!("overflow"),
        };

        unsafe {
            if (*shared).is_unique() {
                let v = &mut (*shared).vec;

                let v_capacity = v.capacity();
                let ptr = v.as_mut_ptr();

                let offset = self.ptr.as_ptr().offset_from(ptr) as usize;

                let new_cap_plus_offset = match new_cap.checked_add(offset) {
                    Some(new_cap_plus_offset) => new_cap_plus_offset,
                    None if !allocate => return false,
                    None => panic!("overflow"),
                };

                if v_capacity >= new_cap_plus_offset {
                    self.cap = new_cap;
                } else if v_capacity >= new_cap && offset >= len {
                    ptr::copy_nonoverlapping(self.ptr.as_ptr(), ptr, len);

                    self.ptr = NonNull::new_unchecked(ptr);
                    self.cap = v.capacity();
                } else {
                    if !allocate {
                        return false;
                    }
                    new_cap = new_cap_plus_offset;
                    let double = v.capacity().checked_shl(1).unwrap_or(new_cap);

                    new_cap = cmp::max(double, new_cap);
                    v.set_len(offset + len);
                    v.reserve(new_cap - v.len());
                    self.ptr = NonNull::new_unchecked(v.as_mut_ptr().add(offset));
                    self.cap = v.capacity() - offset;
                }

                return true;
            }
        }
        if !allocate {
            return false;
        }

        let original_capacity_repr = unsafe { (*shared).original_capacity_repr };
        let original_capacity = original_capacity_from_repr(original_capacity_repr);

        new_cap = cmp::max(new_cap, original_capacity);

        let mut v = ManuallyDrop::new(Vec::with_capacity(new_cap));

        v.extend_from_slice(self.as_ref());

        unsafe { Shared::release_shared(shared) };

        let data = (original_capacity_repr << ORIGINAL_CAPACITY_OFFSET) | Self::KIND_VEC;
        self.data = invalid_ptr(data);
        self.ptr = unsafe { NonNull::new_unchecked(v.as_mut_ptr()) };
        self.cap = v.capacity();
        true
    }
}

unsafe impl Send for BytesMut {}
unsafe impl Sync for BytesMut {}

impl Default for BytesMut {
    #[inline]
    fn default() -> BytesMut {
        BytesMut::new()
    }
}

impl Drop for BytesMut {
    fn drop(&mut self) {
        let kind = self.kind();

        if kind == Self::KIND_VEC {
            unsafe {
                let off = self.get_vec_pos();
                let _ = rebuild_vec(self.ptr.as_ptr(), self.len, self.cap, off);
            }
        } else if kind == Self::KIND_ARC {
            unsafe { Shared::release_shared(self.data) };
        }
    }
}

impl Clone for BytesMut {
    fn clone(&self) -> Self {
        Self::from(&self[..])
    }
}

impl AsRef<[u8]> for BytesMut {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl Deref for BytesMut {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.as_ref()
    }
}

impl AsMut<[u8]> for BytesMut {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_slice_mut()
    }
}

impl DerefMut for BytesMut {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

impl PartialEq for BytesMut {
    fn eq(&self, other: &BytesMut) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl PartialOrd for BytesMut {
    fn partial_cmp(&self, other: &BytesMut) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BytesMut {
    fn cmp(&self, other: &BytesMut) -> cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl Eq for BytesMut {}

impl hash::Hash for BytesMut {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        let s: &[u8] = self.as_ref();
        s.hash(state);
    }
}

impl Borrow<[u8]> for BytesMut {
    fn borrow(&self) -> &[u8] {
        self.as_ref()
    }
}

impl BorrowMut<[u8]> for BytesMut {
    fn borrow_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

impl<'a> From<&'a [u8]> for BytesMut {
    fn from(src: &'a [u8]) -> Self {
        Self::from_vec(src.to_vec())
    }
}

impl<'a> From<&'a str> for BytesMut {
    fn from(src: &'a str) -> Self {
        Self::from(src.as_bytes())
    }
}

impl FromIterator<u8> for BytesMut {
    fn from_iter<T: IntoIterator<Item = u8>>(into_iter: T) -> Self {
        Self::from_vec(Vec::from_iter(into_iter))
    }
}

impl<'a> FromIterator<&'a u8> for BytesMut {
    fn from_iter<T: IntoIterator<Item = &'a u8>>(into_iter: T) -> Self {
        Self::from_iter(into_iter.into_iter().copied())
    }
}

impl<'a> IntoIterator for &'a BytesMut {
    type Item = &'a u8;
    type IntoIter = core::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_ref().iter()
    }
}

impl From<BytesMut> for Bytes {
    fn from(src: BytesMut) -> Bytes {
        src.freeze()
    }
}

impl fmt::Write for BytesMut {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if (isize::MAX as usize - s.len()) >= s.len() {
            self.extend_from_slice(s.as_bytes());
            Ok(())
        } else {
            Err(fmt::Error)
        }
    }

    #[inline]
    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        fmt::write(self, args)
    }
}

struct Shared {
    vec: Vec<u8>,
    original_capacity_repr: usize,
    ref_count: AtomicUsize,
}

impl Shared {
    const VTABLE: Vtable = Vtable {
        clone: Self::shared_v_clone,
        drop: Self::shared_v_drop,
        is_unique: Self::shared_v_is_unique,
        into_mut: Self::shared_v_to_mut,
        into_vec: Self::shared_v_to_vec,
    };

    fn init_to_raw(b: Box<MaybeUninit<Self>>, v: Self) -> *mut Self {
        let shared = Box::into_raw(b).cast::<Self>();
        unsafe { shared.write(v) };
        shared
    }

    fn is_unique(&self) -> bool {
        self.ref_count.load(Ordering::Acquire) == 1
    }

    unsafe fn shared_v_to_mut(shared: *mut (), ptr: *const u8, len: usize) -> BytesMut {
        let shared: *mut Shared = shared.cast();

        if unsafe { (*shared).is_unique() } {
            let shared = unsafe { &mut *shared };
            let v = &mut shared.vec;
            let v_capacity = v.capacity();
            let v_ptr = v.as_mut_ptr();
            let offset = unsafe { ptr.offset_from(v_ptr) as usize };
            let cap = v_capacity - offset;

            let ptr = unsafe { NonNull::new_unchecked(ptr as *mut u8) };

            BytesMut {
                ptr,
                len,
                cap,
                data: shared,
            }
        } else {
            let v = unsafe { slice::from_raw_parts(ptr, len).to_vec() };
            unsafe {
                Self::release_shared(shared);
            }
            BytesMut::from_vec(v)
        }
    }

    unsafe fn shared_v_to_vec(shared: *mut (), ptr: *const u8, len: usize) -> Vec<u8> {
        let shared: *mut Shared = shared.cast();

        if unsafe { (*shared).is_unique() } {
            let shared = unsafe { &mut *shared };

            let mut vec = core::mem::take(&mut shared.vec);
            unsafe {
                Self::release_shared(shared);

                ptr::copy(ptr, vec.as_mut_ptr(), len);
                vec.set_len(len);
            }
            vec
        } else {
            let v = unsafe { slice::from_raw_parts(ptr, len).to_vec() };

            unsafe {
                Self::release_shared(shared);
            }
            v
        }
    }

    unsafe fn increment_shared(ptr: *mut Self) {
        let old_size = unsafe { (*ptr).ref_count.fetch_add(1, Ordering::Relaxed) };

        if old_size > isize::MAX as usize {
            panic!("ref count out of max");
        }
    }

    unsafe fn release_shared(ptr: *mut Self) {
        unsafe {
            if (*ptr).ref_count.fetch_sub(1, Ordering::Release) == 1 {
                (*ptr).ref_count.load(Ordering::Acquire);
                drop(Box::from_raw(ptr));
            }
        }
    }

    unsafe fn shared_v_clone(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
        let shared = data.load(Ordering::Relaxed) as *mut Self;
        unsafe {
            Self::increment_shared(shared);
        }
        let data = AtomicPtr::new(shared as *mut ());
        unsafe { Bytes::with_vtable(ptr, len, data, &Self::VTABLE) }
    }

    unsafe fn shared_v_drop(shared: *mut (), _ptr: *const u8, _len: usize) {
        unsafe {
            Self::release_shared(shared.cast());
        }
    }

    unsafe fn shared_v_is_unique(data: &AtomicPtr<()>) -> bool {
        let shared = data.load(Ordering::Acquire);
        let ref_count = unsafe { (*shared.cast::<Self>()).ref_count.load(Ordering::Relaxed) };
        ref_count == 1
    }
}

const MAX_ORIGINAL_CAPACITY_WIDTH: usize = 17;
const MIN_ORIGINAL_CAPACITY_WIDTH: usize = 10;
const ORIGINAL_CAPACITY_MASK: usize = 0b11100;
const ORIGINAL_CAPACITY_OFFSET: usize = 2;

const VEC_POS_OFFSET: usize = 5;
const MAX_VEC_POS: usize = usize::MAX >> VEC_POS_OFFSET;
const NOT_VEC_POS_MASK: usize = 0b11111;

#[cfg(target_pointer_width = "64")]
const PTR_WIDTH: usize = 64;
#[cfg(target_pointer_width = "32")]
const PTR_WIDTH: usize = 32;

#[inline]
fn invalid_ptr<T>(addr: usize) -> *mut T {
    let ptr = core::ptr::null_mut::<u8>().wrapping_add(addr);
    ptr.cast::<T>()
}

unsafe fn rebuild_vec(ptr: *mut u8, mut len: usize, mut cap: usize, off: usize) -> Vec<u8> {
    let ptr = unsafe { ptr.sub(off) };
    len += off;
    cap += off;

    unsafe { Vec::from_raw_parts(ptr, len, cap) }
}

#[inline]
fn original_capacity_to_repr(cap: usize) -> usize {
    let width = PTR_WIDTH - ((cap >> MIN_ORIGINAL_CAPACITY_WIDTH).leading_zeros() as usize);
    cmp::min(
        width,
        MAX_ORIGINAL_CAPACITY_WIDTH - MIN_ORIGINAL_CAPACITY_WIDTH,
    )
}

fn original_capacity_from_repr(repr: usize) -> usize {
    if repr == 0 {
        0
    } else {
        1 << (repr + (MIN_ORIGINAL_CAPACITY_WIDTH - 1))
    }
}
