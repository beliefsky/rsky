use std::{
    cell::RefCell,
    io,
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    rc::Rc,
    task::{Context, Poll, Waker},
    time::Duration,
};

use crate::{
    driver::{OpCode, Proactor, ProactorBuilder},
    scoped_thread_local,
    thread::{Executor, ExecutorConfig, JoinHandle, SpawnMeta, console},
};

mod future;
pub use future::*;

mod waker;

scoped_thread_local!(static CURRENT_RUNTIME: Runtime);

pub struct Runtime {
    executor: Rc<Executor>,
    driver: Rc<RefCell<Proactor>>,
}

impl Runtime {
    pub fn new() -> io::Result<Self> {
        Self::builder().build()
    }

    pub fn builder() -> RuntimeBuilder {
        RuntimeBuilder::new()
    }

    pub fn with_current<T, F: FnOnce(&Self) -> T>(f: F) -> T {
        if CURRENT_RUNTIME.is_set() {
            CURRENT_RUNTIME.with(f)
        } else {
            not_in_compio_runtime()
        }
    }

    pub fn enter<T, F: FnOnce() -> T>(&self, f: F) -> T {
        CURRENT_RUNTIME.set(self, f)
    }

    pub fn run(&self) -> bool {
        self.executor.tick()
    }

    pub fn waker(&self) -> Waker {
        self.driver.borrow().waker()
    }

    #[track_caller]
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.block_on_at(future, SpawnMeta::capture())
    }

    pub fn block_on_at<F: Future>(&self, future: F, meta: SpawnMeta) -> F::Output {
        let future = console::instrument_block_on(meta, future);

        let result = catch_unwind(AssertUnwindSafe(|| {
            self.enter(|| {
                let waker = self.waker();
                let mut context = Context::from_waker(&waker);
                let mut future = std::pin::pin!(future);
                loop {
                    println!("loop, current task: {}", self.num_alive_tasks());
                    if let Poll::Ready(result) = future.as_mut().poll(&mut context) {
                        self.run();
                        return result;
                    }
                    let remaining_tasks = self.run();
                    if remaining_tasks {
                        self.poll_with(Some(Duration::ZERO));
                    } else {
                        self.poll();
                    }
                }
            })
        }));

        match result {
            Ok(output) => output,
            Err(payload) => {
                self.enter(|| self.executor.clear());
                resume_unwind(payload)
            }
        }
    }

    #[track_caller]
    pub fn spawn<F: Future + 'static>(&self, future: F) -> JoinHandle<F::Output> {
        self.spawn_at(future, SpawnMeta::capture())
    }

    pub fn spawn_at<F: Future + 'static>(
        &self,
        future: F,
        meta: SpawnMeta,
    ) -> JoinHandle<F::Output> {
        self.executor.spawn_at(future, meta)
    }

    pub fn num_alive_tasks(&self) -> usize {
        self.executor.num_alive_tasks()
    }

    pub fn submit<T: OpCode + 'static>(&self, op: T) -> Submit<T> {
        Submit::new(self.driver.clone(), op)
    }

    pub fn current_timeout(&self) -> Option<Duration> {
        let timeout = None;
        timeout
    }

    pub fn poll(&self) {
        let timeout = self.current_timeout();
        self.poll_with(timeout)
    }

    pub fn poll_with(&self, timeout: Option<Duration>) {
        let mut driver = self.driver.borrow_mut();
        match driver.poll(timeout) {
            Ok(()) => {}
            Err(e) => match e.kind() {
                io::ErrorKind::TimedOut | io::ErrorKind::Interrupted => {}
                _ => panic!("{e:?}"),
            },
        }
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        // this is not the last runtime reference, no need to clear
        if Rc::strong_count(&self.executor) > 1 {
            return;
        }

        self.enter(|| {
            self.executor.clear();
        })
    }
}

pub struct RuntimeBuilder {
    proactor_builder: ProactorBuilder,
    local_queue_size: usize,
    event_interval: u32,
}

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        RuntimeBuilder {
            proactor_builder: ProactorBuilder::new(),
            event_interval: 61,
            local_queue_size: 64,
        }
    }

    pub fn build(&self) -> io::Result<Runtime> {
        let RuntimeBuilder {
            proactor_builder,
            local_queue_size,
            event_interval,
        } = self;

        // if !thread_affinity.is_empty() {
        //     bind_to_cpu_set(thread_affinity);
        // }

        let driver = proactor_builder.build()?;
        let executor = Executor::with_config(ExecutorConfig {
            max_interval: *event_interval,
            local_queue_size: *local_queue_size,
            waker: Some(driver.waker()),
        });

        Ok(Runtime {
            executor: Rc::new(executor),
            driver: Rc::new(RefCell::new(driver)),
        })
    }
}

#[track_caller]
pub fn spawn<F: Future + 'static>(future: F) -> JoinHandle<F::Output> {
    let meta = SpawnMeta::capture();
    spawn_at(future, meta)
}

pub fn spawn_at<F: Future + 'static>(future: F, meta: SpawnMeta) -> JoinHandle<F::Output> {
    Runtime::with_current(|r| r.spawn_at(future, meta))
}

pub fn submit<T: OpCode + 'static>(op: T) -> Submit<T> {
    Runtime::with_current(|r| r.submit(op))
}

#[cold]
fn not_in_compio_runtime() -> ! {
    panic!("not in a compio runtime")
}
