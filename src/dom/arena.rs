use crate::dom::node::Node;

use std::cell::{Cell, RefCell, Ref};
use std::ops::Deref;


thread_local! {
    static ARENA: RefCell<Vec<Node>> = RefCell::new(Vec::new());
}

// TODO: the way we implemented this with the guard is correct, however instead of having the guard
// on the entire arena we should rather have the guard on only the one node, this will allow for
// multiple references to different nodes inside the same arena.
pub struct NodeRef {
    id: usize,
    guard: RefCell<Option<Ref<'static, Node>>>,
}

impl Deref for NodeRef {
    type Target = Node;

    fn deref(&self) -> &Node {
        /*
        if self.guard.borrow().is_none() {
            let arena = ARENA.with(|arena| unsafe { std::mem::transmute::<Ref<'_, Vec<Node>>, Ref<'static, Vec<Node>>>(arena.borrow()) });
            let node = Ref::map(arena, |arena| &arena[self.id]);

            *self.guard.borrow_mut() = Some(node);
        }

        unsafe {
            (*self.guard.as_ptr()).as_ref().unwrap().deref()
        }
        */

        todo!()
    }
}


/*
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
*/


