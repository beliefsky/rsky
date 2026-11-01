use std::io;

use super::SEntry;
use crate::driver::Extra;

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
    /// Create submission entry.
    fn create_entry(&mut self) -> OpEntry;

    unsafe fn set_result(&mut self, _: &io::Result<usize>, _: &Extra) {}
}
