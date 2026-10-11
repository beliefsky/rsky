mod buf_result;
mod io_buf;
mod io_vec_buf;

pub use buf_result::*;
pub use io_buf::*;
pub use io_vec_buf::*;

pub trait IntoInner {
    type Inner;

    fn into_inner(self) -> Self::Inner;
}
