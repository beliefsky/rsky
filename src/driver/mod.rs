mod cancel;
mod control;
mod fd;
mod key;
mod sys;

pub use cancel::*;
pub use fd::*;
pub use key::Key;
pub use sys::*;

use crate::io::{self, BufResult};
use core::{
    task::{Poll, Waker},
    time::Duration,
};
use std::os::fd::RawFd;

pub enum PushEntry<K, R> {
    Pending(K),
    Ready(R),
}

impl<K, R> PushEntry<K, R> {
    pub const fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }

    pub fn take_ready(self) -> Option<R> {
        match self {
            Self::Pending(_) => None,
            Self::Ready(res) => Some(res),
        }
    }

    pub fn map_pending<L>(self, f: impl FnOnce(K) -> L) -> PushEntry<L, R> {
        match self {
            Self::Pending(k) => PushEntry::Pending(f(k)),
            Self::Ready(r) => PushEntry::Ready(r),
        }
    }

    pub fn map_ready<S>(self, f: impl FnOnce(R) -> S) -> PushEntry<K, S> {
        match self {
            Self::Pending(k) => PushEntry::Pending(k),
            Self::Ready(r) => PushEntry::Ready(f(r)),
        }
    }
}

pub struct Proactor {
    driver: Driver,
}

impl Proactor {
    pub fn new() -> io::Result<Self> {
        Self::builder().build()
    }

    pub fn builder() -> ProactorBuilder {
        ProactorBuilder::new()
    }

    pub fn with_builder(builder: &ProactorBuilder) -> io::Result<Self> {
        Ok(Self {
            driver: Driver::new(builder)?,
        })
    }

    pub fn default_extra(&self) -> Extra {
        Extra::new(&self.driver)
    }
    pub fn attach(&mut self, fd: RawFd) -> io::Result<()> {
        self.driver.attach(fd)
    }

    pub fn push<T: sys::OpCode + 'static>(
        &mut self,
        op: T,
    ) -> PushEntry<Key<T>, BufResult<usize, T>> {
        self.push_with_extra(op, self.default_extra())
    }

    pub fn push_with_extra<T: sys::OpCode + 'static>(
        &mut self,
        op: T,
        extra: Extra,
    ) -> PushEntry<Key<T>, BufResult<usize, T>> {
        let key = Key::new(op, extra);
        match self.driver.push(key.clone().erase()) {
            Poll::Pending => PushEntry::Pending(key),
            Poll::Ready(res) => {
                key.set_result(res);
                PushEntry::Ready(key.take_result())
            }
        }
    }

    pub fn poll(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        self.driver.poll(timeout)
    }

    pub fn pop<T: OpCode>(&mut self, key: Key<T>) -> PushEntry<Key<T>, BufResult<usize, T>> {
        if key.has_result() {
            let (res, buf) = key.take_result().into_parts();
            PushEntry::Ready(BufResult::new(res, buf))
        } else {
            PushEntry::Pending(key)
        }
    }

    pub fn pop_with_extra<T: OpCode>(
        &mut self,
        key: Key<T>,
    ) -> PushEntry<Key<T>, (BufResult<usize, T>, Extra)> {
        if key.has_result() {
            let extra = key.swap_extra(self.default_extra());
            let (res, buf) = key.take_result().into_parts();
            PushEntry::Ready((BufResult::new(res, buf), extra))
        } else {
            PushEntry::Pending(key)
        }
    }

    pub fn cancel<T: OpCode>(&mut self, key: Key<T>) -> Option<BufResult<usize, T>> {
        if key.set_cancelled() {
            return None;
        }
        if key.is_unique() && key.has_result() {
            let (res, buf) = key.take_result().into_parts();
            Some(BufResult::new(res, buf))
        } else {
            self.driver.cancel(key.erase());
            None
        }
    }

    pub fn cancel_token(&mut self, token: Cancel) -> bool {
        let Some(key) = token.upgrade() else {
            return false;
        };
        if key.set_cancelled() || key.has_result() {
            return false;
        }
        self.driver.cancel(key);
        true
    }

    pub fn register_cancel<T: OpCode>(&mut self, key: &Key<T>) -> Cancel {
        Cancel::new(key)
    }

    pub fn update_waker<T>(&mut self, op: &Key<T>, waker: &Waker) {
        op.set_waker(waker);
    }

    pub fn waker(&self) -> Waker {
        self.driver.waker()
    }
}

pub struct ProactorBuilder {
    capacity: u32,
    sqpoll_idle: Option<Duration>,
    sqpoll_cpu: Option<u32>,
    cqsize: Option<u32>,
    single_issuer: bool,
    coop_taskrun: bool,
    taskrun_flag: bool,
    defer_taskrun: bool,
}

impl ProactorBuilder {
    pub fn new() -> Self {
        ProactorBuilder {
            capacity: 1024,
            sqpoll_idle: None,
            sqpoll_cpu: None,
            cqsize: None,
            single_issuer: false,
            coop_taskrun: false,
            taskrun_flag: false,
            defer_taskrun: false,
        }
    }

    pub fn build(&self) -> io::Result<Proactor> {
        Proactor::with_builder(self)
    }

    pub fn capacity(&mut self, capacity: u32) -> &mut Self {
        self.capacity = capacity;
        self
    }
    pub fn cqsize(&mut self, cqsize: u32) -> &mut Self {
        self.cqsize = Some(cqsize);
        self
    }
    pub fn sqpoll_idle(&mut self, idle: Duration) -> &mut Self {
        self.sqpoll_idle = Some(idle);
        self
    }
    pub fn sqpoll_cpu(&mut self, cpu: u32) -> &mut Self {
        self.sqpoll_cpu = Some(cpu);
        self
    }
    pub fn single_issuer(&mut self, enable: bool) -> &mut Self {
        self.single_issuer = enable;
        self
    }
    pub fn coop_taskrun(&mut self, enable: bool) -> &mut Self {
        self.coop_taskrun = enable;
        self
    }
    pub fn taskrun_flag(&mut self, enable: bool) -> &mut Self {
        self.taskrun_flag = enable;
        self
    }
    pub fn defer_taskrun(&mut self, enable: bool) -> &mut Self {
        self.defer_taskrun = enable;
        self
    }
}
