use super::BytesMut;

use core::{
    cmp, hash,
    mem::{self, ManuallyDrop, MaybeUninit},
    ops::{Deref, RangeBounds},
    ptr, slice,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

use alloc::{
    alloc::{Layout, dealloc},
    borrow::Borrow,
    boxed::Box,
};

pub struct Bytes {
    ptr: *const u8,
    len: usize,
    data: AtomicPtr<()>,
    vtable: &'static Vtable,
}

impl Bytes {
    /// # Examples
    ///
    /// ```
    /// use rsky::io::Bytes;
    ///
    /// let b = Bytes::new();
    /// assert_eq!(&b[..], b"");
    /// ```
    #[inline]
    pub const fn new() -> Self {
        const EMPTY: &[u8] = &[];
        Bytes::from_static(EMPTY)
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::Bytes;
    ///
    /// let b = Bytes::from_static(b"hello");
    /// assert_eq!(&b[..], b"hello");
    /// ```
    #[inline]
    pub const fn from_static(bytes: &'static [u8]) -> Self {
        Bytes {
            ptr: bytes.as_ptr(),
            len: bytes.len(),
            data: AtomicPtr::new(ptr::null_mut()),
            vtable: &Vtable::STATIC_VTABLE,
        }
    }

    pub fn from_owner<T>(owner: T) -> Self
    where
        T: AsRef<[u8]> + Send + 'static,
    {
        let owned = Box::into_raw(Box::new(Owned {
            ref_cnt: AtomicUsize::new(1),
            owner,
        }));

        let mut ret = Bytes {
            ptr: ptr::NonNull::dangling().as_ptr(),
            len: 0,
            data: AtomicPtr::new(owned.cast()),
            vtable: &Owned::<T>::VTABLE,
        };

        let buf = unsafe { &*owned }.owner.as_ref();
        ret.ptr = buf.as_ptr();
        ret.len = buf.len();

        ret
    }

    pub fn copy_from_slice(data: &[u8]) -> Self {
        data.to_vec().into()
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::Bytes;
    ///
    /// let a = Bytes::from(&b"hello world"[..]);
    /// let b = a.slice(2..5);
    ///
    /// assert_eq!(&b[..], b"llo");
    /// ```
    pub fn slice(&self, range: impl RangeBounds<usize>) -> Self {
        let (begin, end) = super::range(range, self.len());

        if end == begin {
            return Self::new_empty_with_ptr(self.ptr.wrapping_add(begin));
        }

        let mut ret = self.clone();

        ret.len = end - begin;
        ret.ptr = unsafe { ret.ptr.add(begin) };

        ret
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::Bytes;
    ///
    /// let bytes = Bytes::from(&b"012345678"[..]);
    /// let as_slice = bytes.as_ref();
    /// let subset = &as_slice[2..6];
    /// let subslice = bytes.slice_ref(&subset);
    /// assert_eq!(&subslice[..], b"2345");
    /// ```
    ///
    pub fn slice_ref(&self, subset: &[u8]) -> Self {
        if subset.is_empty() {
            return Bytes::new();
        }

        let bytes_p = self.as_ptr() as usize;

        let sub_p = subset.as_ptr() as usize;
        let sub_len = subset.len();

        let sub_offset = sub_p - bytes_p;

        self.slice(sub_offset..(sub_offset + sub_len))
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::Bytes;
    ///
    /// let mut a = Bytes::from(&b"hello world"[..]);
    /// let b = a.split_off(5);
    ///
    /// assert_eq!(&a[..], b"hello");
    /// assert_eq!(&b[..], b" world");
    /// ```
    pub fn split_off(&mut self, at: usize) -> Self {
        if at == self.len() {
            return Bytes::new_empty_with_ptr(self.ptr.wrapping_add(at));
        }

        if at == 0 {
            return mem::replace(self, Bytes::new_empty_with_ptr(self.ptr));
        }
        let mut ret = self.clone();

        self.len = at;

        unsafe { ret.inc_start(at) };

        ret
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::Bytes;
    ///
    /// let mut a = Bytes::from(&b"hello world"[..]);
    /// let b = a.split_to(5);
    ///
    /// assert_eq!(&a[..], b" world");
    /// assert_eq!(&b[..], b"hello");
    /// ```
    pub fn split_to(&mut self, at: usize) -> Self {
        if at == self.len() {
            let end_ptr = self.ptr.wrapping_add(at);
            return mem::replace(self, Bytes::new_empty_with_ptr(end_ptr));
        }

        if at == 0 {
            return Bytes::new_empty_with_ptr(self.ptr);
        }
        let mut ret = self.clone();

        unsafe { self.inc_start(at) };

        ret.len = at;
        ret
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::Bytes;
    ///
    /// let mut buf = Bytes::from(&b"hello world"[..]);
    /// buf.truncate(5);
    /// assert_eq!(buf, b"hello"[..]);
    /// ```
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        if len == 0 {
            drop(mem::replace(self, Bytes::new_empty_with_ptr(self.ptr)));
        } else if len < self.len {
            if self.vtable as *const Vtable == &Vtable::PROMOTABLE_EVEN_VTABLE
                || self.vtable as *const Vtable == &Vtable::PROMOTABLE_ODD_VTABLE
            {
                drop(self.split_off(len));
            } else {
                self.len = len;
            }
        }
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::Bytes;
    ///
    /// let mut buf = Bytes::from(&b"hello world"[..]);
    /// buf.clear();
    /// assert!(buf.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.truncate(0);
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::Bytes;
    ///
    /// let b = Bytes::from(&b"hello"[..]);
    /// assert_eq!(b.len(), 5);
    /// ```
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::Bytes;
    ///
    /// let b = Bytes::new();
    /// assert!(b.is_empty());
    /// ```
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::Bytes;
    ///
    /// let a = Bytes::from(vec![1, 2, 3]);
    /// assert!(a.is_unique());
    /// let b = a.clone();
    /// assert!(!a.is_unique());
    /// ```
    pub fn is_unique(&self) -> bool {
        unsafe { (self.vtable.is_unique)(&self.data) }
    }

    /// # Examples
    ///
    /// ```
    /// use rsky::io::{Bytes, BytesMut};
    ///
    /// let bytes = Bytes::from(b"hello".to_vec());
    /// assert_eq!(bytes.try_into_mut(), Ok(BytesMut::from(&b"hello"[..])));
    /// ```
    pub fn try_into_mut(self) -> Result<BytesMut, Bytes> {
        if self.is_unique() {
            Ok(self.into())
        } else {
            Err(self)
        }
    }

    #[inline]
    pub(super) unsafe fn with_vtable(
        ptr: *const u8,
        len: usize,
        data: AtomicPtr<()>,
        vtable: &'static Vtable,
    ) -> Bytes {
        Bytes {
            ptr,
            len,
            data,
            vtable,
        }
    }

    #[inline]
    pub(super) unsafe fn inc_start(&mut self, by: usize) {
        self.len -= by;
        self.ptr = unsafe { self.ptr.add(by) };
    }

    #[inline]
    fn new_empty_with_ptr(ptr: *const u8) -> Self {
        let ptr = ptr::null::<u8>().wrapping_add(ptr as usize);

        Bytes {
            ptr,
            len: 0,
            data: AtomicPtr::new(ptr::null_mut()),
            vtable: &Vtable::STATIC_VTABLE,
        }
    }

    #[inline]
    fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }

    #[inline]
    fn data_mut(&mut self) -> *mut () {
        *(self.data.get_mut())
    }
}

impl Default for Bytes {
    #[inline]
    fn default() -> Bytes {
        Bytes::new()
    }
}

unsafe impl Send for Bytes {}
unsafe impl Sync for Bytes {}

impl Drop for Bytes {
    #[inline]
    fn drop(&mut self) {
        let data = self.data_mut();
        unsafe { (self.vtable.drop)(data, self.ptr, self.len) }
    }
}

impl Clone for Bytes {
    #[inline]
    fn clone(&self) -> Bytes {
        unsafe { (self.vtable.clone)(&self.data, self.ptr, self.len) }
    }
}

impl Deref for Bytes {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl AsRef<[u8]> for Bytes {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl hash::Hash for Bytes {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.as_slice().hash(state);
    }
}

impl Borrow<[u8]> for Bytes {
    fn borrow(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<'a> IntoIterator for &'a Bytes {
    type Item = &'a u8;
    type IntoIter = core::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

// impl Eq

impl PartialEq for Bytes {
    fn eq(&self, other: &Bytes) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl PartialOrd for Bytes {
    fn partial_cmp(&self, other: &Bytes) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Bytes {
    fn cmp(&self, other: &Bytes) -> cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl Eq for Bytes {}

impl PartialEq<[u8]> for Bytes {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_slice() == other
    }
}

impl PartialOrd<[u8]> for Bytes {
    fn partial_cmp(&self, other: &[u8]) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other)
    }
}

impl PartialEq<Bytes> for [u8] {
    fn eq(&self, other: &Bytes) -> bool {
        *other == *self
    }
}

impl PartialOrd<Bytes> for [u8] {
    fn partial_cmp(&self, other: &Bytes) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

impl PartialEq<str> for Bytes {
    fn eq(&self, other: &str) -> bool {
        self.as_slice() == other.as_bytes()
    }
}

impl PartialOrd<str> for Bytes {
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_bytes())
    }
}

impl PartialEq<Bytes> for str {
    fn eq(&self, other: &Bytes) -> bool {
        *other == *self
    }
}

impl PartialOrd<Bytes> for str {
    fn partial_cmp(&self, other: &Bytes) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self.as_bytes(), other)
    }
}

impl PartialEq<Vec<u8>> for Bytes {
    fn eq(&self, other: &Vec<u8>) -> bool {
        *self == other[..]
    }
}

impl PartialOrd<Vec<u8>> for Bytes {
    fn partial_cmp(&self, other: &Vec<u8>) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(&other[..])
    }
}

impl PartialEq<Bytes> for Vec<u8> {
    fn eq(&self, other: &Bytes) -> bool {
        *other == *self
    }
}

impl PartialOrd<Bytes> for Vec<u8> {
    fn partial_cmp(&self, other: &Bytes) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

impl PartialEq<String> for Bytes {
    fn eq(&self, other: &String) -> bool {
        *self == other[..]
    }
}

impl PartialOrd<String> for Bytes {
    fn partial_cmp(&self, other: &String) -> Option<cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_bytes())
    }
}

impl PartialEq<Bytes> for String {
    fn eq(&self, other: &Bytes) -> bool {
        *other == *self
    }
}

impl PartialOrd<Bytes> for String {
    fn partial_cmp(&self, other: &Bytes) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self.as_bytes(), other)
    }
}

impl PartialEq<Bytes> for &[u8] {
    fn eq(&self, other: &Bytes) -> bool {
        *other == *self
    }
}

impl PartialOrd<Bytes> for &[u8] {
    fn partial_cmp(&self, other: &Bytes) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self, other)
    }
}

impl PartialEq<Bytes> for &str {
    fn eq(&self, other: &Bytes) -> bool {
        *other == *self
    }
}

impl PartialOrd<Bytes> for &str {
    fn partial_cmp(&self, other: &Bytes) -> Option<cmp::Ordering> {
        <[u8] as PartialOrd<[u8]>>::partial_cmp(self.as_bytes(), other)
    }
}

impl<'a, T: ?Sized> PartialEq<&'a T> for Bytes
where
    Bytes: PartialEq<T>,
{
    fn eq(&self, other: &&'a T) -> bool {
        *self == **other
    }
}

impl<'a, T: ?Sized> PartialOrd<&'a T> for Bytes
where
    Bytes: PartialOrd<T>,
{
    fn partial_cmp(&self, other: &&'a T) -> Option<cmp::Ordering> {
        self.partial_cmp(&**other)
    }
}

// impl From

impl From<&'static [u8]> for Bytes {
    fn from(slice: &'static [u8]) -> Self {
        Self::from_static(slice)
    }
}

impl From<&'static str> for Bytes {
    fn from(slice: &'static str) -> Self {
        Self::from_static(slice.as_bytes())
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(vec: Vec<u8>) -> Self {
        if vec.len() == vec.capacity() {
            return Self::from(vec.into_boxed_slice());
        }

        let shared = Box::new(MaybeUninit::<Shared>::uninit());
        let mut vec = ManuallyDrop::new(vec);
        let ptr = vec.as_mut_ptr();
        let len = vec.len();
        let cap = vec.capacity();

        let shared = Shared::init_to_raw(
            shared,
            Shared {
                buf: ptr,
                cap,
                ref_cnt: AtomicUsize::new(1),
            },
        );
        Self {
            ptr,
            len,
            data: AtomicPtr::new(shared as _),
            vtable: &Shared::VTABLE,
        }
    }
}

impl From<Box<[u8]>> for Bytes {
    fn from(slice: Box<[u8]>) -> Self {
        if slice.is_empty() {
            return Self::new();
        }

        let len = slice.len();
        let ptr = Box::into_raw(slice) as *mut u8;

        if ptr as usize & 0x1 == 0 {
            let data = ptr_map(ptr, |addr| addr | Vtable::KIND_VEC);
            Self {
                ptr,
                len,
                data: AtomicPtr::new(data.cast()),
                vtable: &Vtable::PROMOTABLE_EVEN_VTABLE,
            }
        } else {
            Self {
                ptr,
                len,
                data: AtomicPtr::new(ptr.cast()),
                vtable: &Vtable::PROMOTABLE_ODD_VTABLE,
            }
        }
    }
}

impl From<String> for Bytes {
    fn from(s: String) -> Self {
        Self::from(s.into_bytes())
    }
}

impl From<Bytes> for BytesMut {
    /// # Examples
    ///
    /// ```
    /// use rsky::io::{Bytes, BytesMut};
    ///
    /// let bytes = Bytes::from(b"hello".to_vec());
    /// assert_eq!(BytesMut::from(bytes), BytesMut::from(&b"hello"[..]));
    /// ```
    fn from(bytes: Bytes) -> Self {
        let mut bytes = ManuallyDrop::new(bytes);
        let data = bytes.data_mut();
        unsafe { (bytes.vtable.into_mut)(data, bytes.ptr, bytes.len) }
    }
}

impl From<Bytes> for Vec<u8> {
    fn from(bytes: Bytes) -> Vec<u8> {
        let mut bytes = ManuallyDrop::new(bytes);
        let data = bytes.data_mut();
        unsafe { (bytes.vtable.into_vec)(data, bytes.ptr, bytes.len) }
    }
}

// ===== impl Vtable =====

pub(super) struct Vtable {
    pub clone: unsafe fn(&AtomicPtr<()>, *const u8, usize) -> Bytes,

    pub drop: unsafe fn(*mut (), *const u8, usize),

    pub is_unique: unsafe fn(&AtomicPtr<()>) -> bool,

    pub into_mut: unsafe fn(*mut (), *const u8, usize) -> BytesMut,

    pub into_vec: unsafe fn(*mut (), *const u8, usize) -> Vec<u8>,
}

impl Vtable {
    const KIND_ARC: usize = 0b0;
    const KIND_VEC: usize = 0b1;
    const KIND_MASK: usize = 0b1;

    const STATIC_VTABLE: Self = Self {
        clone: Self::static_clone,
        drop: Self::static_drop,
        is_unique: Self::static_is_unique,
        into_mut: Self::static_to_mut,
        into_vec: Self::static_to_vec,
    };

    const PROMOTABLE_EVEN_VTABLE: Self = Self {
        clone: Self::promotable_even_clone,
        drop: Self::promotable_even_drop,
        is_unique: Self::promotable_is_unique,
        into_mut: Self::promotable_even_to_mut,
        into_vec: Self::promotable_even_to_vec,
    };

    const PROMOTABLE_ODD_VTABLE: Self = Self {
        clone: Self::promotable_odd_clone,
        drop: Self::promotable_odd_drop,
        is_unique: Self::promotable_is_unique,
        into_mut: Self::promotable_odd_to_mut,
        into_vec: Self::promotable_odd_to_vec,
    };

    unsafe fn static_clone(_: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
        let slice = unsafe { slice::from_raw_parts(ptr, len) };
        Bytes::from_static(slice)
    }

    unsafe fn static_drop(_: *mut (), _: *const u8, _: usize) {}

    fn static_is_unique(_: &AtomicPtr<()>) -> bool {
        false
    }

    unsafe fn static_to_mut(_: *mut (), ptr: *const u8, len: usize) -> BytesMut {
        let slice = unsafe { slice::from_raw_parts(ptr, len) };
        BytesMut::from(slice)
    }

    unsafe fn static_to_vec(_: *mut (), ptr: *const u8, len: usize) -> Vec<u8> {
        let slice = unsafe { slice::from_raw_parts(ptr, len) };
        slice.to_vec()
    }

    unsafe fn promotable_even_clone(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
        let shared = data.load(Ordering::Acquire);
        let kind = shared as usize & Self::KIND_MASK;

        if kind == Self::KIND_ARC {
            unsafe { Shared::shallow_clone_arc(shared.cast(), ptr, len) }
        } else {
            let buf = ptr_map(shared.cast(), |addr| addr & !Self::KIND_MASK);
            unsafe { Shared::shallow_clone_vec(data, shared, buf, ptr, len) }
        }
    }

    unsafe fn promotable_odd_clone(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
        let shared = data.load(Ordering::Acquire);
        let kind = shared as usize & Self::KIND_MASK;

        if kind == Self::KIND_ARC {
            unsafe { Shared::shallow_clone_arc(shared as _, ptr, len) }
        } else {
            debug_assert_eq!(kind, Self::KIND_VEC);
            unsafe { Shared::shallow_clone_vec(data, shared, shared.cast(), ptr, len) }
        }
    }

    unsafe fn promotable_even_drop(shared: *mut (), ptr: *const u8, len: usize) {
        let kind = shared as usize & Self::KIND_MASK;

        if kind == Self::KIND_ARC {
            unsafe {
                Shared::release_shared(shared.cast());
            }
        } else {
            let buf = ptr_map(shared.cast(), |addr| addr & !Self::KIND_MASK);
            unsafe {
                Self::free_boxed_slice(buf, ptr, len);
            }
        }
    }

    unsafe fn promotable_odd_drop(shared: *mut (), ptr: *const u8, len: usize) {
        let kind = shared as usize & Self::KIND_MASK;

        if kind == Self::KIND_ARC {
            unsafe {
                Shared::release_shared(shared.cast());
            }
        } else {
            unsafe {
                Self::free_boxed_slice(shared.cast(), ptr, len);
            }
        }
    }

    unsafe fn promotable_is_unique(data: &AtomicPtr<()>) -> bool {
        let shared = data.load(Ordering::Acquire);
        let kind = shared as usize & Self::KIND_MASK;

        if kind == Self::KIND_ARC {
            let ref_cnt = unsafe { (*shared.cast::<Shared>()).ref_cnt.load(Ordering::Relaxed) };
            ref_cnt == 1
        } else {
            true
        }
    }

    unsafe fn promotable_even_to_mut(shared: *mut (), ptr: *const u8, len: usize) -> BytesMut {
        unsafe {
            Self::promotable_to_mut(shared, ptr, len, |shared| {
                ptr_map(shared.cast(), |addr| addr & !Self::KIND_MASK)
            })
        }
    }
    unsafe fn promotable_odd_to_mut(shared: *mut (), ptr: *const u8, len: usize) -> BytesMut {
        unsafe { Self::promotable_to_mut(shared, ptr, len, |shared| shared.cast()) }
    }

    unsafe fn promotable_even_to_vec(shared: *mut (), ptr: *const u8, len: usize) -> Vec<u8> {
        unsafe {
            Self::promotable_to_vec(shared, ptr, len, |shared| {
                ptr_map(shared.cast(), |addr| addr & !Self::KIND_MASK)
            })
        }
    }

    unsafe fn promotable_odd_to_vec(shared: *mut (), ptr: *const u8, len: usize) -> Vec<u8> {
        unsafe { Self::promotable_to_vec(shared, ptr, len, |shared| shared.cast()) }
    }

    unsafe fn free_boxed_slice(buf: *mut u8, offset: *const u8, len: usize) {
        unsafe {
            let cap = offset.offset_from(buf) as usize + len;
            dealloc(buf, Layout::from_size_align(cap, 1).unwrap())
        }
    }

    unsafe fn promotable_to_mut(
        shared: *mut (),
        ptr: *const u8,
        len: usize,
        f: fn(*mut ()) -> *mut u8,
    ) -> BytesMut {
        let kind = shared as usize & Self::KIND_MASK;

        if kind == Self::KIND_ARC {
            unsafe { Shared::shared_to_mut_impl(shared.cast(), ptr, len) }
        } else {
            let buf = f(shared);
            let off = unsafe { ptr.offset_from(buf) as usize };
            let cap = off + len;
            let v = unsafe { Vec::from_raw_parts(buf, cap, cap) };

            let mut b = BytesMut::from_vec(v);
            unsafe {
                b.advance_unchecked(off);
            }
            b
        }
    }

    unsafe fn promotable_to_vec(
        shared: *mut (),
        ptr: *const u8,
        len: usize,
        f: fn(*mut ()) -> *mut u8,
    ) -> Vec<u8> {
        let kind = shared as usize & Self::KIND_MASK;

        if kind == Self::KIND_ARC {
            unsafe { Shared::shared_to_vec_impl(shared.cast(), ptr, len) }
        } else {
            let buf = f(shared);
            let cap = unsafe { ptr.offset_from(buf) as usize + len };

            unsafe {
                ptr::copy(ptr, buf, len);
                Vec::from_raw_parts(buf, len, cap)
            }
        }
    }
}

// ===== impl OwnedVtable =====

#[repr(C)]
struct Owned<T> {
    ref_cnt: AtomicUsize,
    owner: T,
}

impl<T> Owned<T> {
    const VTABLE: Vtable = Vtable {
        clone: Self::owned_clone,
        drop: Self::owned_drop,
        is_unique: Self::owned_is_unique,
        into_mut: Self::owned_to_mut,
        into_vec: Self::owned_to_vec,
    };

    unsafe fn owned_clone(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
        let owned = data.load(Ordering::Relaxed);
        let old_cnt = unsafe { (*owned.cast::<AtomicUsize>()).fetch_add(1, Ordering::Relaxed) };
        if old_cnt > usize::MAX >> 1 {
            panic!("ref count out of max");
        }

        Bytes {
            ptr,
            len,
            data: AtomicPtr::new(owned as _),
            vtable: &Owned::<T>::VTABLE,
        }
    }

    unsafe fn owned_drop(data: *mut (), _ptr: *const u8, _len: usize) {
        unsafe {
            Self::owned_drop_impl(data);
        }
    }

    unsafe fn owned_drop_impl(owned: *mut ()) {
        {
            let ref_cnt = unsafe { &*owned.cast::<AtomicUsize>() };

            let old_cnt = ref_cnt.fetch_sub(1, Ordering::Release);
            if old_cnt == 1 {
                ref_cnt.load(Ordering::Acquire);
            }
        }

        drop(unsafe { Box::<Owned<T>>::from_raw(owned.cast()) });
    }

    unsafe fn owned_is_unique(_data: &AtomicPtr<()>) -> bool {
        false
    }

    unsafe fn owned_to_mut(owned: *mut (), ptr: *const u8, len: usize) -> BytesMut {
        unsafe { BytesMut::from_vec(Self::owned_to_vec(owned, ptr, len)) }
    }

    unsafe fn owned_to_vec(owned: *mut (), ptr: *const u8, len: usize) -> Vec<u8> {
        let slice = unsafe { slice::from_raw_parts(ptr, len) };
        let vec = slice.to_vec();
        unsafe { Self::owned_drop_impl(owned) };
        vec
    }
}

// ===== impl SharedVtable =====

struct Shared {
    buf: *mut u8,
    cap: usize,
    ref_cnt: AtomicUsize,
}

impl Shared {
    const VTABLE: Vtable = Vtable {
        clone: Self::shared_clone,
        drop: Self::shared_drop,
        is_unique: Self::shared_is_unique,
        into_mut: Self::shared_to_mut,
        into_vec: Self::shared_to_vec,
    };

    fn init_to_raw(b: Box<MaybeUninit<Self>>, v: Self) -> *mut Self {
        let shared = Box::into_raw(b).cast::<Self>();
        unsafe { shared.write(v) };
        shared
    }

    unsafe fn shared_clone(data: &AtomicPtr<()>, ptr: *const u8, len: usize) -> Bytes {
        let shared = data.load(Ordering::Relaxed);
        unsafe { Self::shallow_clone_arc(shared as _, ptr, len) }
    }

    unsafe fn shared_drop(shared: *mut (), _ptr: *const u8, _len: usize) {
        unsafe {
            Self::release_shared(shared.cast());
        }
    }

    unsafe fn shared_to_mut(shared: *mut (), ptr: *const u8, len: usize) -> BytesMut {
        unsafe { Self::shared_to_mut_impl(shared.cast(), ptr, len) }
    }

    unsafe fn shared_to_vec(shared: *mut (), ptr: *const u8, len: usize) -> Vec<u8> {
        unsafe { Self::shared_to_vec_impl(shared.cast(), ptr, len) }
    }

    unsafe fn shared_is_unique(data: &AtomicPtr<()>) -> bool {
        let shared = data.load(Ordering::Acquire);
        let ref_cnt = unsafe { (*shared.cast::<Self>()).ref_cnt.load(Ordering::Relaxed) };
        ref_cnt == 1
    }

    unsafe fn shallow_clone_arc(shared: *mut Self, ptr: *const u8, len: usize) -> Bytes {
        let old_size = unsafe { (*shared).ref_cnt.fetch_add(1, Ordering::Relaxed) };

        if old_size > usize::MAX >> 1 {
            panic!("ref count out of max");
        }

        Bytes {
            ptr,
            len,
            data: AtomicPtr::new(shared as _),
            vtable: &Self::VTABLE,
        }
    }

    unsafe fn shallow_clone_vec(
        atom: &AtomicPtr<()>,
        ptr: *const (),
        buf: *mut u8,
        offset: *const u8,
        len: usize,
    ) -> Bytes {
        let shared = Box::new(MaybeUninit::<Self>::uninit());
        let shared = Self::init_to_raw(
            shared,
            Self {
                buf,
                cap: unsafe { offset.offset_from(buf) as usize } + len,
                ref_cnt: AtomicUsize::new(2),
            },
        );
        match atom.compare_exchange(ptr as _, shared as _, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => Bytes {
                ptr: offset,
                len,
                data: AtomicPtr::new(shared as _),
                vtable: &Self::VTABLE,
            },
            Err(actual) => {
                let shared = unsafe { Box::from_raw(shared) };
                mem::forget(*shared);
                unsafe { Self::shallow_clone_arc(actual as _, offset, len) }
            }
        }
    }

    unsafe fn release_shared(ptr: *mut Self) {
        unsafe {
            if (*ptr).ref_cnt.fetch_sub(1, Ordering::Release) == 1 {
                (*ptr).ref_cnt.load(Ordering::Acquire);
                drop(Box::from_raw(ptr));
            }
        }
    }

    unsafe fn shared_to_mut_impl(shared: *mut Shared, ptr: *const u8, len: usize) -> BytesMut {
        if unsafe { (*shared).ref_cnt.load(Ordering::Acquire) } == 1 {
            let shared = unsafe { *Box::from_raw(shared) };
            let shared = ManuallyDrop::new(shared);
            let buf = shared.buf;
            let cap = shared.cap;

            let off = unsafe { ptr.offset_from(buf) as usize };
            let v = unsafe { Vec::from_raw_parts(buf, len + off, cap) };

            let mut b = BytesMut::from_vec(v);
            unsafe {
                b.advance_unchecked(off);
            }
            b
        } else {
            let v = unsafe { slice::from_raw_parts(ptr, len).to_vec() };
            unsafe {
                Self::release_shared(shared);
            }
            BytesMut::from_vec(v)
        }
    }

    unsafe fn shared_to_vec_impl(shared: *mut Shared, ptr: *const u8, len: usize) -> Vec<u8> {
        if unsafe {
            (*shared)
                .ref_cnt
                .compare_exchange(1, 0, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
        } {
            let shared = unsafe { *Box::from_raw(shared) };
            let shared = ManuallyDrop::new(shared);
            let buf = shared.buf;
            let cap = shared.cap;

            unsafe {
                ptr::copy(ptr, buf, len);
                Vec::from_raw_parts(buf, len, cap)
            }
        } else {
            let v = unsafe { slice::from_raw_parts(ptr, len).to_vec() };
            unsafe {
                Self::release_shared(shared);
            }
            v
        }
    }
}

impl Drop for Shared {
    fn drop(&mut self) {
        unsafe { dealloc(self.buf, Layout::from_size_align(self.cap, 1).unwrap()) }
    }
}

fn ptr_map<F>(ptr: *mut u8, f: F) -> *mut u8
where
    F: FnOnce(usize) -> usize,
{
    let old_addr = ptr as usize;
    let new_addr = f(old_addr);
    new_addr as *mut u8
}
