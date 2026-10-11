use crate::driver::Driver;

mod iour;
use iour as sys;

pub(in crate::driver::sys) use sys::*;

#[repr(transparent)]
pub struct Extra(pub(super) sys::Extra);

impl<I: Into<sys::Extra>> From<I> for Extra {
    fn from(inner: I) -> Self {
        Self(inner.into())
    }
}

impl Extra {
    pub(crate) fn new(driver: &Driver) -> Self {
        driver.default_extra().into()
    }
}
