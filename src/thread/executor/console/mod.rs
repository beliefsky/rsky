mod disabled;
pub use disabled::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum WakerOp {
    Clone,
    Drop,
    Wake,
    WakeByRef,
}
