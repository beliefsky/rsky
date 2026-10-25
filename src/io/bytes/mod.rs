mod bytes;
mod bytes_mut;

pub use bytes::Bytes;
pub use bytes_mut::BytesMut;

use bytes::Vtable;
use core::ops::{Bound, RangeBounds};

#[inline(always)]
fn range(range: impl RangeBounds<usize>, len: usize) -> (usize, usize) {
    let begin = match range.start_bound() {
        Bound::Included(&n) => n,
        Bound::Excluded(&n) => n.checked_add(1).expect("out of range"),
        Bound::Unbounded => 0,
    };

    let end = match range.end_bound() {
        Bound::Included(&n) => n.checked_add(1).expect("out of range"),
        Bound::Excluded(&n) => n,
        Bound::Unbounded => len,
    };
    (begin, end)
}
