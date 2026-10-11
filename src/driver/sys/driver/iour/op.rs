use std::io;

use super::SEntry;
use crate::driver::{Extra, control::Carrier};

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
                // Set the union of two flags - it will not remove previous flags set by the Op
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

    /// 初始化控件
    unsafe fn init(&mut self, _: &mut Self::Control) {}

    /// Create submission entry.
    fn create_entry(&mut self, _: &mut Self::Control) -> OpEntry;

    unsafe fn set_result(&mut self, _: &mut Self::Control, _: &io::Result<usize>, _: &Extra) {}
}

pub(crate) trait Carry {
    /// See [`OpCode::create_entry`].
    fn create_entry(&mut self) -> OpEntry;

    /// See [`OpCode::set_result`].
    unsafe fn set_result(&mut self, _: &io::Result<usize>, _: &Extra);
}

impl<T: OpCode> Carry for Carrier<T> {
    fn create_entry(&mut self) -> OpEntry {
        let (op, control) = (&mut self.op, &mut self.control);
        op.create_entry(control)
    }

    unsafe fn set_result(&mut self, result: &io::Result<usize>, extra: &Extra) {
        let (op, control) = (&mut self.op, &mut self.control);
        unsafe { OpCode::set_result(op, control, result, extra) }
    }
}
