//! No-op stand-ins used when the `console` feature is disabled.

use core::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
pub struct SpawnMeta;

impl SpawnMeta {
    #[inline(always)]
    pub fn capture() -> Self {
        Self
    }

    #[inline(always)]
    pub fn named(self, _name: &'static str) -> Self {
        self
    }

    #[inline(always)]
    pub fn untracked() -> Self {
        Self
    }
}

#[must_use = "the task span is exited as soon as this is dropped"]
pub(crate) struct Entered<'a>(PhantomData<&'a TaskSpan>);

pub(crate) type EnterGuard<'a> = Entered<'a>;

#[derive(Debug)]
pub(crate) struct TaskSpan;

impl TaskSpan {
    #[inline(always)]
    #[expect(
        clippy::extra_unused_type_parameters,
        reason = "mirrors the enabled variant, which records the future's size"
    )]
    pub(crate) fn new<F>(_meta: SpawnMeta) -> Self {
        Self
    }

    #[inline(always)]
    pub(crate) fn enter(&self) -> EnterGuard<'_> {
        Entered(PhantomData)
    }

    #[inline(always)]
    pub(crate) fn waker_op(&self, _op: super::WakerOp) {}
}

#[inline(always)]
pub fn instrument_blocking<T, F: FnOnce() -> T>(_meta: SpawnMeta, f: F) -> impl FnOnce() -> T {
    f
}

#[inline(always)]
pub fn instrument_block_on<F: Future>(_meta: SpawnMeta, fut: F) -> impl Future<Output = F::Output> {
    fut
}

#[inline(always)]
pub fn instrument_execute<F: Future>(_meta: SpawnMeta, fut: F) -> impl Future<Output = F::Output> {
    fut
}
