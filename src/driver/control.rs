use super::OpCode;

pub(crate) struct Carrier<T: OpCode + ?Sized> {
    control: T::Control,
    op: T,
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

    #[inline]
    pub fn into_inner(self) -> T {
        self.op
    }
}
