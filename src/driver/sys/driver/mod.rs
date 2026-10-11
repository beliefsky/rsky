use std::sync::atomic::{AtomicU8, Ordering};

mod iour;
pub use iour::*;

struct AwakeFlag(AtomicU8);

impl AwakeFlag {
    const IDLE: u8 = 0b00;
    const NOTIFIED: u8 = 0b01;
    const AWAKE: u8 = 0b10;

    pub fn new() -> Self {
        Self(AtomicU8::new(Self::IDLE))
    }

    pub fn set(&self) {
        self.0.store(Self::AWAKE, Ordering::Release);
    }

    pub fn reset(&self) -> bool {
        (self.0.swap(Self::IDLE, Ordering::AcqRel) & Self::NOTIFIED) != 0
    }

    pub fn wake(&self) -> bool {
        self.0.fetch_or(Self::NOTIFIED, Ordering::AcqRel) != 0
    }
}
