use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::{
    driver::{Key, OpCode, Proactor},
    runtime::Runtime,
};

struct Inner {
    // tokens: RefCell<HashSet<Cancel>>,
    is_cancelled: Cell<bool>,
    driver: Rc<RefCell<Proactor>>,
    // notify: Event,
}

#[derive(Clone)]
pub struct CancelToken(Rc<Inner>);

impl PartialEq for CancelToken {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for CancelToken {}

impl CancelToken {
    pub fn new() -> Self {
        Self(Rc::new(Inner {
            // tokens: RefCell::new(HashSet::new()),
            is_cancelled: Cell::new(false),
            driver: Runtime::with_current(|r| r.driver.clone()),
            // notify: Event::new(),
        }))
    }

    pub fn register<T: OpCode>(&self, key: &Key<T>) {
        println!("CancelToken register未实现");
    }
}

impl Default for CancelToken {
    fn default() -> Self {
        Self::new()
    }
}
