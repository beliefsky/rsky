use core::{cell::RefCell, task::Waker};

#[repr(transparent)]
pub struct WakerSlot {
    waker: RefCell<Option<Waker>>,
}

impl WakerSlot {
    pub const fn new() -> Self {
        Self {
            waker: RefCell::new(None),
        }
    }

    /// 生产者将Waker放进AtomicWaker容器
    pub fn register(&self, waker: &Waker) {
        let mut w = self.waker.borrow_mut();
        if w.as_ref().is_some_and(|x| x.will_wake(waker)) {
            return;
        }
        *w = Some(waker.clone())
    }

    /// 消费者取出并调用
    pub fn wake(&self) {
        if let Some(waker) = self.take() {
            waker.wake()
        }
    }

    /// 消费者从容器中取出Waker
    pub fn take(&self) -> Option<Waker> {
        self.waker.borrow_mut().take()
    }
}
