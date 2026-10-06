use crate::{buf::IntoInner, driver::OpCode};

type ControlInner<T> = <T as OpCode>::Control;

pub(crate) struct Carrier<T: OpCode + ?Sized> {
    pub(crate) control: ControlInner<T>,
    pub(crate) op: T,
}

impl<T: OpCode> Carrier<T> {
    pub fn new(op: T) -> Self {
        let control = T::Control::default();
        Self { control, op }
    }

    pub unsafe fn init(&mut self) {
        unsafe { OpCode::init(&mut self.op, &mut self.control) }
    }
}

impl<T: OpCode> IntoInner for Carrier<T> {
    type Inner = T;

    fn into_inner(self) -> Self::Inner {
        self.op
    }
}
