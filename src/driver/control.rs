use crate::{driver::OpCode, io::IntoInner};

pub(crate) struct Carrier<T: OpCode + ?Sized> {
    pub(crate) op: T,
}

impl<T: OpCode> Carrier<T> {
    pub fn new(op: T) -> Self {
        Self { op }
    }
}

impl<T: OpCode> IntoInner for Carrier<T> {
    type Inner = T;

    fn into_inner(self) -> Self::Inner {
        self.op
    }
}
