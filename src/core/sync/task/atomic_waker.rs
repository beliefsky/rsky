use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicUsize, Ordering},
    task::Waker,
};

pub struct AtomicWaker {
    state: AtomicUsize,
    waker: UnsafeCell<Option<Waker>>,
}

impl AtomicWaker {
    const WAITING: usize = 0;
    const REGISTERING: usize = 0b01;
    const WAKING: usize = 0b10;

    pub const fn new() -> Self {
        Self {
            state: AtomicUsize::new(Self::WAITING),
            waker: UnsafeCell::new(None),
        }
    }

    /// 生产者将Waker放进AtomicWaker容器
    pub fn register(&self, waker: &Waker) {
        match self
            .state
            .compare_exchange(
                Self::WAITING,
                Self::REGISTERING,
                Ordering::Acquire,
                Ordering::Acquire,
            )
            .unwrap_or_else(|x| x)
        {
            Self::WAITING => unsafe {
                match &*self.waker.get() {
                    Some(old_waker) if old_waker.will_wake(waker) => (),
                    _ => *self.waker.get() = Some(waker.clone()),
                }

                let res = self.state.compare_exchange(
                    Self::REGISTERING,
                    Self::WAITING,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                );

                match res {
                    Ok(_) => {}
                    Err(actual) => {
                        debug_assert_eq!(actual, Self::REGISTERING | Self::WAKING);

                        let waker = (*self.waker.get()).take().unwrap();

                        self.state.swap(Self::WAITING, Ordering::AcqRel);

                        waker.wake();
                    }
                }
            },

            Self::WAKING => {
                waker.wake_by_ref();
            }
            state => {
                debug_assert!(
                    state == Self::REGISTERING || state == Self::REGISTERING | Self::WAKING
                );
            }
        }
    }

    /// 消费者取出并调用
    pub fn wake(&self) {
        if let Some(waker) = self.take() {
            waker.wake();
        }
    }

    /// 消费者从容器中取出Waker
    pub fn take(&self) -> Option<Waker> {
        match self.state.fetch_or(Self::WAKING, Ordering::AcqRel) {
            Self::WAITING => {
                let waker = unsafe { (*self.waker.get()).take() };

                let old_state = self.state.swap(Self::WAITING, Ordering::Release);
                debug_assert!(old_state == Self::WAKING);

                waker
            }
            state => {
                debug_assert!(
                    state == Self::REGISTERING
                        || state == Self::REGISTERING | Self::WAKING
                        || state == Self::WAKING
                );
                None
            }
        }
    }
}

impl Default for AtomicWaker {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for AtomicWaker {}
unsafe impl Sync for AtomicWaker {}
