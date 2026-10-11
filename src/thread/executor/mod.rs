use super::SendWrapper;
use crossbeam_queue::SegQueue;
use queue::{TaskId, TaskQueue};
use std::{
    any::Any,
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
    task::Waker,
};

pub mod console;
mod join_handle;
mod queue;
mod task;
mod util;
mod waker;

pub use console::SpawnMeta;
pub use join_handle::{JoinError, JoinHandle, ResumeUnwind};
use util::panic_guard;

#[repr(transparent)]
struct UnsafeCell<T>(std::cell::UnsafeCell<T>);

impl<T> UnsafeCell<T> {
    pub fn new(value: T) -> Self {
        Self(std::cell::UnsafeCell::new(value))
    }

    #[inline(always)]
    pub fn with_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(*mut T) -> R,
    {
        f(self.0.get())
    }

    #[inline(always)]
    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(*const T) -> R,
    {
        f(self.0.get())
    }
}

pub(crate) type PanicResult<T> = Result<T, Panic>;
pub(crate) type Panic = Box<dyn Any + Send + 'static>;

pub struct Executor {
    ptr: NonNull<Shared>,
    config: ExecutorConfig,
}

/// Configuration for [`Executor`].
#[derive(Clone)]
pub struct ExecutorConfig {
    pub local_queue_size: usize,

    pub max_interval: u32,

    pub waker: Option<Waker>,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            local_queue_size: 64,
            max_interval: 61,
            waker: None,
        }
    }
}

pub(crate) struct Shared {
    waker: Option<Waker>,
    sync: SegQueue<TaskId>,
    pending: AtomicUsize,
    queue: SendWrapper<TaskQueue>,
}

impl Shared {
    #[inline]
    pub(crate) fn drain_sync(&self, queue: &TaskQueue) {
        if self.pending.load(Ordering::Acquire) == 0 {
            return;
        }

        let mut drained: usize = 0;
        while let Some(id) = self.sync.pop() {
            queue.make_hot(id);
            drained += 1;
        }

        if drained != 0 {
            self.pending.fetch_sub(drained, Ordering::Release);
        }
    }
}

impl Executor {
    pub fn new() -> Self {
        Self::with_config(ExecutorConfig::default())
    }

    pub fn with_config(mut config: ExecutorConfig) -> Self {
        let ptr = Box::into_raw(Box::new(Shared {
            waker: config.waker.take(),
            sync: SegQueue::new(),
            pending: AtomicUsize::new(0),
            queue: SendWrapper::new(TaskQueue::new(config.local_queue_size)),
        }));

        Self {
            config,
            ptr: unsafe { NonNull::new_unchecked(ptr) },
        }
    }

    #[track_caller]
    pub fn spawn<F: Future + 'static>(&self, fut: F) -> JoinHandle<F::Output> {
        self.spawn_at(fut, SpawnMeta::capture())
    }

    pub fn spawn_at<F: Future + 'static>(&self, fut: F, meta: SpawnMeta) -> JoinHandle<F::Output> {
        let shared = self.shared();
        let tracker = shared.queue.tracker();
        // SAFETY: Executor cannot be sent to ther thread
        let queue = unsafe { shared.queue.get_unchecked() };
        let task = queue.insert(self.ptr, tracker, fut, meta);

        JoinHandle::new(task)
    }

    pub fn tick(&self) -> bool {
        let queue = self.queue();

        self.shared().drain_sync(queue);

        for id in queue.iter_hot().take(self.config.max_interval as _) {
            queue.make_cold(id);
            let task = queue.take(id).expect("Task was not reset back");
            let res = unsafe { task.run() };
            if res.is_ready() {
                unsafe { task.drop() };
                queue.remove(id);
            } else {
                queue.reset(id, task);
            }
        }

        queue.has_hot()
    }

    pub fn has_task(&self) -> bool {
        self.queue().hot_head().is_some()
    }

    pub fn num_alive_tasks(&self) -> usize {
        self.queue().len()
    }

    pub fn clear(&self) {
        while self.shared().sync.pop().is_some() {}
        unsafe { self.queue().clear() };
    }

    #[inline(always)]
    fn shared(&self) -> &Shared {
        unsafe { self.ptr.as_ref() }
    }

    #[inline(always)]
    fn queue(&self) -> &TaskQueue {
        unsafe { self.shared().queue.get_unchecked() }
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        self.clear();
        unsafe { drop(Box::from_raw(self.ptr.as_ptr())) };
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}
