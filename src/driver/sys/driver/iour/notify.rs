use std::{
    io,
    sync::Arc,
    task::{Wake, Waker},
};

use crate::driver::sys::driver::AwakeFlag;

pub(super) struct Notifier {
    notify: Arc<Notify>,
}

impl Notifier {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            notify: Arc::new(Notify::new()),
        })
    }

    pub fn set_awake(&self) {
        self.notify.set_awake();
    }

    pub fn reset(&self) -> bool {
        self.notify.reset()
    }

    pub fn waker(&self) -> Waker {
        Waker::from(self.notify.clone())
    }
}

pub(super) struct Notify {
    // fd: OwnedFd,
    awake: AwakeFlag,
}

impl Notify {
    pub fn new() -> Self {
        Self {
            awake: AwakeFlag::new(),
        }
    }

    pub fn set_awake(&self) {
        self.awake.set();
    }

    pub fn reset(&self) -> bool {
        self.awake.reset()
    }
}

impl Wake for Notify {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        println!("==========> wake_by_ref");
        if self.awake.wake() {
            println!("==========> event write");
            // 写入通知
        }
    }
}
