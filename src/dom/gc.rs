use std::cell::RefCell;
use std::rc::{Rc, Weak};


/// OwnedDom is a owning reference to a DOM object.
pub type OwnedDom<T> = Rc<RefCell<T>>;

/// WeakDom is a non-owning reference to a DOM object.
pub struct WeakDom<T> {
    pub(crate) inner: Weak<RefCell<T>>,
}

impl<T> Clone for WeakDom<T> {
    fn clone(&self) -> WeakDom<T> {
        WeakDom {
            inner: Weak::clone(&self.inner),
        }
    }
}

impl<T> WeakDom<T> {
    pub fn new_from_owned(inner: OwnedDom<T>) -> WeakDom<T> {
        WeakDom {
            inner: Rc::downgrade(&inner),
        }
    }

    pub fn upgrade(&self) -> OwnedDom<T> {
        self.inner.upgrade().expect("DOM object doesnt exist")
    }
}


