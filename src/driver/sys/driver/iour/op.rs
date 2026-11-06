use super::SEntry;
use crate::{
    driver::{Extra, control::Carrier},
    io,
};

pub enum OpEntry {
    /// This operation creates an io-uring submission entry.
    Submission(SEntry),
}

impl OpEntry {
    pub(crate) fn with_extra(self, extra: &Extra) -> Self {
        match self {
            Self::Submission(mut entry) => Self::Submission({
                if let Some(personality) = extra.get_personality() {
                    entry = entry.personality(personality);
                }
                entry.flags(extra.get_sqe_flags())
            }),
        }
    }
}

impl From<SEntry> for OpEntry {
    fn from(value: SEntry) -> Self {
        Self::Submission(value)
    }
}

pub unsafe trait OpCode {
    type Control: Default;

    unsafe fn init(&mut self, _: &mut Self::Control) {}

    fn create_entry(&mut self, _: &mut Self::Control) -> OpEntry;

    unsafe fn set_result(&mut self, _: &mut Self::Control, _: &io::Result<usize>, _: &Extra) {}
}

pub(crate) trait Carry {
    fn create_entry(&mut self) -> OpEntry;

    unsafe fn set_result(&mut self, _: &io::Result<usize>, _: &Extra);
}

impl<T: OpCode> Carry for Carrier<T> {
    fn create_entry(&mut self) -> OpEntry {
        let (op, control) = self.inner_mut();
        op.create_entry(control)
    }

    unsafe fn set_result(&mut self, result: &io::Result<usize>, extra: &Extra) {
        let (op, control) = self.inner_mut();
        unsafe { OpCode::set_result(op, control, result, extra) }
    }
}
