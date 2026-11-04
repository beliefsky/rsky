use crate::error::Error;

use super::IntoInner;

pub struct BufResult<T, B>(Result<T, Error>, B);

impl<T, B> BufResult<T, B> {
    #[inline]
    pub fn new(res: Result<T, Error>, buf: B) -> Self {
        Self(res, buf)
    }

    pub const fn is_ok(&self) -> bool {
        self.0.is_ok()
    }

    pub const fn is_err(&self) -> bool {
        self.0.is_err()
    }

    #[inline]
    pub fn res(&self) -> &Result<T, Error> {
        &self.0
    }

    #[inline]
    pub fn res_mut(&mut self) -> &mut Result<T, Error> {
        &mut self.0
    }
    #[inline]
    pub fn buf(&self) -> &B {
        &self.1
    }

    #[inline]
    pub fn buf_mut(&mut self) -> &mut B {
        &mut self.1
    }

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

    #[inline]
    pub fn map_with_buf<U, C>(
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

    #[inline]
    pub fn map_res<U>(self, f: impl FnOnce(T) -> U) -> BufResult<U, B> {
        BufResult(self.0.map(f), self.1)
    }

    #[inline]
    pub fn map_buf<C>(self, f: impl FnOnce(B) -> C) -> BufResult<T, C> {
        BufResult(self.0, f(self.1))
    }

    #[inline]
    pub fn and_then<U>(self, f: impl FnOnce(T, B) -> (Result<U, Error>, B)) -> BufResult<U, B> {
        match self.0 {
            Ok(res) => BufResult::from(f(res, self.1)),
            Err(e) => BufResult(Err(e), self.1),
        }
    }

    #[inline]
    #[track_caller]
    pub fn expect(self, msg: &str) -> (T, B) {
        (self.0.expect(msg), self.1)
    }

    #[inline(always)]
    #[track_caller]
    pub fn unwrap(self) -> (T, B) {
        (self.0.unwrap(), self.1)
    }

    #[inline]
    pub fn into_res(self) -> Result<T, Error> {
        self.0
    }

    #[inline]
    pub fn into_parts(self) -> (Result<T, Error>, B) {
        (self.0, self.1)
    }
}

impl<T, B> From<(Result<T, Error>, B)> for BufResult<T, B> {
    fn from((res, buf): (Result<T, Error>, B)) -> Self {
        Self::new(res, buf)
    }
}

impl<T, B> From<BufResult<T, B>> for (Result<T, Error>, B) {
    fn from(res: BufResult<T, B>) -> Self {
        res.into_parts()
    }
}

impl<T: IntoInner, O> IntoInner for BufResult<O, T> {
    type Inner = BufResult<O, T::Inner>;

    fn into_inner(self) -> Self::Inner {
        BufResult::new(self.0, self.1.into_inner())
    }
}
