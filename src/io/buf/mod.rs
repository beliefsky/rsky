mod buf_result;
mod io_buf;

pub use buf_result::BufResult;
pub use io_buf::{IoBuf, IoBufMut};

pub trait IntoInner {
    type Inner;

    fn into_inner(self) -> Self::Inner;
}
