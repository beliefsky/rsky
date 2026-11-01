use core::marker::PhantomData;

#[derive(Clone, Copy)]
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

pub(crate) struct Entered<'a>(PhantomData<&'a TaskSpan>);

pub(crate) type EnterGuard<'a> = Entered<'a>;

pub(crate) struct TaskSpan;

impl TaskSpan {
    #[inline(always)]
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
