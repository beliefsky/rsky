use super::OpCode;
use crate::io::IntoInner;

pub(crate) struct Carrier<T: OpCode + ?Sized> {
    control: T::Control,
    op: T,
}

impl<T: OpCode> IntoInner for Carrier<T> {
    type Inner = T;

    #[inline]
    fn into_inner(self) -> Self::Inner {
        self.op
    }
}

impl<T: OpCode> Carrier<T> {
    pub fn new(op: T) -> Self {
        let control = T::Control::default();
        Self { control, op }
    }

    pub unsafe fn init(&mut self) {
        unsafe { OpCode::init(&mut self.op, &mut self.control) }
    }

    pub fn inner_mut(&mut self) -> (&mut T, &mut T::Control) {
        (&mut self.op, &mut self.control)
    }
}
