mod io_buf;
pub use io_buf::*;

mod buf_result;
pub use buf_result::BufResult;

pub trait IntoInner {
    /// The inner type.
    type Inner;

    /// Get the inner buffer.
    fn into_inner(self) -> Self::Inner;
}
