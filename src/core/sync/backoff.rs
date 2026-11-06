use core::{cell::Cell, hint};

const SPIN_LIMIT: u32 = 6;
const YIELD_LIMIT: u32 = 10;

/// 自旋锁策略
#[repr(transparent)]
pub struct Backoff {
    step: Cell<u32>,
}

impl Backoff {
    #[inline]
    pub fn new() -> Self {
        Backoff { step: Cell::new(0) }
    }

    #[inline]
    pub fn reset(&self) {
        self.step.set(0);
    }

    pub fn spin(&self) {
        for _ in 0..1 << self.step.get().min(SPIN_LIMIT) {
            hint::spin_loop();
        }

        if self.step.get() <= SPIN_LIMIT {
            self.step.set(self.step.get() + 1);
        }
    }

    pub fn snooze(&self) {
        if self.step.get() <= SPIN_LIMIT {
            self.spin_loop();
        } else {
            #[cfg(not(any(unix, windows)))]
            self.spin_loop();

            #[cfg(any(unix, windows))]
            ::std::thread::yield_now();
        }

        if self.step.get() <= YIELD_LIMIT {
            self.step.set(self.step.get() + 1);
        }
    }

    #[inline]
    pub fn is_completed(&self) -> bool {
        self.step.get() > YIELD_LIMIT
    }

    fn spin_loop(&self) {
        for _ in 0..1 << self.step.get() {
            hint::spin_loop();
        }
    }
}

impl Default for Backoff {
    fn default() -> Backoff {
        Backoff::new()
    }
}
