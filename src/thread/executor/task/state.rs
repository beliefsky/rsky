use std::{
    process::abort,
    sync::atomic::{
        AtomicUsize,
        Ordering::{self, *},
    },
};

/// Executor has scheduled the task to run, but it is not running.
const SCHEDULED: usize = 1 << 0;

/// A waker is scheduling from remote.
const SCHEDULING: usize = 1 << 1;

/// Not in the process of cloning and setting a waker to the task.
const NOT_SETTING_WAKER: usize = 1 << 2;

/// A waker is set to the task.
const HAS_WAKER: usize = 1 << 3;

/// The task is completed, and `FutureState` has switched to `result`.
const COMPLETED: usize = 1 << 4;

/// The result is present.
const HAS_RESULT: usize = 1 << 5;

/// The task is not cancelled. It might be cancelled because either the
/// `JoinHandle` is dropped without detaching or the Exector is dropped.
const NOT_CANCELLED: usize = 1 << 6;

const RC_SHIFT: usize = 7;
const RC_UNIT: usize = 1 << RC_SHIFT;
const RC_MAX: usize = usize::MAX >> RC_SHIFT;

#[repr(transparent)]
pub(crate) struct State(AtomicUsize);

#[derive(Clone, Copy)]
#[repr(transparent)]
pub(crate) struct Snapshot(usize);

pub(super) trait Consistency {
    const ACQUIRE: Ordering;
    const ACQ_REL: Ordering;
    const RELEASE: Ordering;
}

pub(super) struct Strong;
pub(super) struct Weak;

impl Consistency for Strong {
    const ACQUIRE: Ordering = Acquire;
    const ACQ_REL: Ordering = AcqRel;
    const RELEASE: Ordering = Release;
}

impl Consistency for Weak {
    const ACQUIRE: Ordering = Relaxed;
    const ACQ_REL: Ordering = Relaxed;
    const RELEASE: Ordering = Relaxed;
}

impl State {
    /// Create a state with two given count.
    pub(crate) fn new<const N: usize>() -> Self {
        const INIT: usize = NOT_SETTING_WAKER | NOT_CANCELLED;

        Self(AtomicUsize::new((N * RC_UNIT) | INIT))
    }

    pub(crate) fn set_has_result<C: Consistency, const SET: bool>(&self) {
        if SET {
            self.0.fetch_or(HAS_RESULT, C::RELEASE);
        } else {
            self.0.fetch_and(!HAS_RESULT, C::RELEASE);
        }
    }

    pub(crate) fn set_has_waker<C: Consistency, const SET: bool>(&self) {
        if SET {
            self.0.fetch_or(HAS_WAKER, C::RELEASE);
        } else {
            self.0.fetch_and(!HAS_WAKER, C::RELEASE);
        }
    }

    pub(crate) fn start_scheduling(&self) -> Snapshot {
        Snapshot(self.0.fetch_or(SCHEDULED | SCHEDULING, Strong::ACQ_REL))
    }

    pub(crate) fn finish_scheduling(&self) {
        self.0.fetch_and(!SCHEDULING, Strong::RELEASE);
    }

    pub(crate) fn unschedule(&self) -> Snapshot {
        Snapshot(self.0.fetch_and(!SCHEDULED, Strong::ACQ_REL))
    }

    pub(crate) fn set_cancelled(&self) -> Snapshot {
        Snapshot(self.0.fetch_and(!NOT_CANCELLED, Strong::ACQ_REL))
    }

    pub(crate) fn finish_running(&self) -> Snapshot {
        Snapshot(self.0.fetch_or(COMPLETED | HAS_RESULT, Strong::ACQ_REL))
    }

    pub(crate) fn start_setting_waker(&self) -> Snapshot {
        Snapshot(self.0.fetch_and(!NOT_SETTING_WAKER, Strong::ACQ_REL))
    }

    pub(crate) fn finish_setting_waker<const SUCCESS: bool>(&self) -> Snapshot {
        let flag = if SUCCESS {
            NOT_SETTING_WAKER | HAS_WAKER
        } else {
            NOT_SETTING_WAKER
        };

        Snapshot(self.0.fetch_or(flag, Strong::ACQ_REL))
    }

    /// Mark as no waker & cancelled
    pub(crate) fn set_dropped(&self) -> Snapshot {
        const FLAG: usize = !HAS_WAKER & !NOT_CANCELLED;

        Snapshot(self.0.fetch_and(FLAG, Strong::ACQ_REL))
    }

    /// Load the state with acquire ordering.
    pub(crate) fn load<C: Consistency>(&self) -> Snapshot {
        Snapshot(self.0.load(C::ACQUIRE))
    }

    pub(crate) fn inc(&self) -> Snapshot {
        let state = Snapshot(self.0.fetch_add(RC_UNIT, Strong::RELEASE));
        if state.count() == RC_MAX {
            abort()
        }
        state
    }

    /// Decrease the reference count by one and return the old state.
    pub(crate) fn dec(&self) -> Snapshot {
        let state = Snapshot(self.0.fetch_sub(RC_UNIT, Strong::ACQ_REL));
        debug_assert!(state.count() >= 1, "Reference count underflow");
        state
    }
}

impl Snapshot {
    pub(crate) fn is_scheduled(&self) -> bool {
        self.0 & SCHEDULED != 0
    }

    pub(crate) fn is_scheduling(&self) -> bool {
        self.0 & SCHEDULING != 0
    }

    pub(crate) fn is_completed(&self) -> bool {
        self.0 & COMPLETED != 0
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.0 & NOT_CANCELLED == 0
    }

    pub(crate) fn is_setting_waker(&self) -> bool {
        self.0 & NOT_SETTING_WAKER == 0
    }

    pub(crate) fn has_waker(&self) -> bool {
        self.0 & HAS_WAKER != 0
    }

    pub(crate) fn has_result(&self) -> bool {
        self.0 & HAS_RESULT != 0
    }

    pub(crate) fn count(&self) -> usize {
        self.0 >> RC_SHIFT
    }
}
