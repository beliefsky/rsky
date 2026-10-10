use std::task::Waker;

use crate::{
    driver::{Extra, Key, OpCode, Proactor, PushEntry},
    io::BufResult,
};

mod combinator;
pub(crate) use combinator::*;

mod future;
pub use future::*;

fn poll_task<T: OpCode>(
    driver: &mut Proactor,
    waker: &Waker,
    key: Key<T>,
) -> PushEntry<Key<T>, BufResult<usize, T>> {
    driver.pop(key).map_pending(|k| {
        driver.update_waker(&k, waker);
        k
    })
}

fn poll_task_with_extra<T: OpCode>(
    driver: &mut Proactor,
    waker: &Waker,
    key: Key<T>,
) -> PushEntry<Key<T>, (BufResult<usize, T>, Extra)> {
    driver.pop_with_extra(key).map_pending(|k| {
        driver.update_waker(&k, waker);
        k
    })
}

fn submit_raw<T: OpCode + 'static>(
    driver: &mut Proactor,
    op: T,
    extra: Option<Extra>,
) -> PushEntry<Key<T>, BufResult<usize, T>> {
    match extra {
        Some(e) => driver.push_with_extra(op, e),
        None => driver.push(op),
    }
}
