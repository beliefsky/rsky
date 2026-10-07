use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    rc::Rc,
};

use crate::{
    driver::{Cancel, Key, OpCode, Proactor},
    runtime::Runtime,
};

struct Inner {
    tokens: RefCell<HashSet<Cancel>>,
    is_cancelled: Cell<bool>,
    driver: Rc<RefCell<Proactor>>,
    // notify: Event,
}

#[derive(Clone)]
pub struct CancelToken(Rc<Inner>);

impl CancelToken {
    pub fn new() -> Self {
        Self(Rc::new(Inner {
            tokens: RefCell::new(HashSet::new()),
            is_cancelled: Cell::new(false),
            driver: Runtime::with_current(|r| r.driver.clone()),
            // notify: Event::new(),
        }))
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.is_cancelled.get()
    }

    pub fn register<T: OpCode>(&self, key: &Key<T>) {
        if self.0.is_cancelled.get() {
            self.0.driver.borrow_mut().cancel(key.clone());
        } else {
            let token = self.0.driver.borrow_mut().register_cancel(key);
            self.0.tokens.borrow_mut().insert(token);
        }
    }
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for CancelToken {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for CancelToken {}
