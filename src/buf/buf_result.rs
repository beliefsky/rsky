use std::io;

use super::IntoInner;

pub struct BufResult<T, B>(pub io::Result<T>, pub B);

impl<T, B> BufResult<T, B> {
    /// Returns [`true`] if the result is [`Ok`].
    pub const fn is_ok(&self) -> bool {
        self.0.is_ok()
    }

    /// Returns [`true`] if the result is [`Err`].
    pub const fn is_err(&self) -> bool {
        self.0.is_err()
    }

    /// Maps the result part, and allows updating the buffer.
    #[inline]
    pub fn map<U>(self, f: impl FnOnce(T, B) -> (U, B)) -> BufResult<U, B> {
        match self.0 {
            Ok(res) => {
                let (res, buf) = f(res, self.1);
                BufResult(Ok(res), buf)
            }
            Err(e) => BufResult(Err(e), self.1),
        }
    }

    /// Maps the result part, and allows changing the buffer type.
    #[inline]
    pub fn map2<U, C>(
        self,
        f_ok: impl FnOnce(T, B) -> (U, C),
        f_err: impl FnOnce(B) -> C,
    ) -> BufResult<U, C> {
        match self.0 {
            Ok(res) => {
                let (res, buf) = f_ok(res, self.1);
                BufResult(Ok(res), buf)
            }
            Err(e) => BufResult(Err(e), f_err(self.1)),
        }
    }

    /// Maps the result part, and keeps the buffer unchanged.
    #[inline]
    pub fn map_res<U>(self, f: impl FnOnce(T) -> U) -> BufResult<U, B> {
        BufResult(self.0.map(f), self.1)
    }

    /// Maps the buffer part, and keeps the result unchanged.
    #[inline]
    pub fn map_buffer<C>(self, f: impl FnOnce(B) -> C) -> BufResult<T, C> {
        BufResult(self.0, f(self.1))
    }

    /// Updating the result type and modifying the buffer.
    #[inline]
    pub fn and_then<U>(self, f: impl FnOnce(T, B) -> (io::Result<U>, B)) -> BufResult<U, B> {
        match self.0 {
            Ok(res) => BufResult::from(f(res, self.1)),
            Err(e) => BufResult(Err(e), self.1),
        }
    }

    /// Returns the contained [`Ok`] value, consuming the `self` value.
    #[inline]
    #[track_caller]
    pub fn expect(self, msg: &str) -> (T, B) {
        (self.0.expect(msg), self.1)
    }

    /// Returns the contained [`Ok`] value, consuming the `self` value.
    #[inline(always)]
    #[track_caller]
    pub fn unwrap(self) -> (T, B) {
        (self.0.unwrap(), self.1)
    }

    /// Returns the parts, consuming the `self` value.
    #[inline]
    pub fn into_parts(self) -> (io::Result<T>, B) {
        (self.0, self.1)
    }
}

impl<T, B> From<(io::Result<T>, B)> for BufResult<T, B> {
    fn from((res, buf): (io::Result<T>, B)) -> Self {
        Self(res, buf)
    }
}

impl<T, B> From<BufResult<T, B>> for (io::Result<T>, B) {
    fn from(BufResult(res, buf): BufResult<T, B>) -> Self {
        (res, buf)
    }
}

impl<T: IntoInner, O> IntoInner for BufResult<O, T> {
    type Inner = BufResult<O, T::Inner>;

    fn into_inner(self) -> Self::Inner {
        BufResult(self.0, self.1.into_inner())
    }
}
