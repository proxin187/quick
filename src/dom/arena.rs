use crate::dom::node::Node;

use std::cell::{RefCell, Ref};
use std::borrow::Borrow;


thread_local! {
    static ARENA: RefCell<Vec<RefCell<Node>>> = RefCell::new(Vec::new());
}

/// NodeId is an index to a node inside the arena.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeId(usize);

/// Insert a node into the arena.
#[inline]
pub fn insert(node: Node) -> NodeId {
    ARENA.with_borrow_mut(|arena| {
        arena.push(RefCell::new(node));

        NodeId(arena.len() - 1)
    })
}

/// Insert a cyclic node into the arena.
#[inline]
pub fn insert_cyclic(f: impl Fn(NodeId) -> Node) -> NodeId {
    ARENA.with_borrow_mut(|arena| {
        arena.push(RefCell::new(f(NodeId(arena.len()))));

        NodeId(arena.len() - 1)
    })
}

/// Get a reference to a node from the arena.
#[inline]
pub fn get<T: Borrow<NodeId>>(id: T) -> Ref<'static, Node> {
    ARENA.with_borrow(|arena| unsafe {
        std::mem::transmute::<Ref<'_, Node>, Ref<'static, Node>>(arena[id.borrow().0].borrow())
    })
}

/// Safely mutate a node from the arena.
#[inline]
pub fn with_mut<A: Borrow<NodeId>, B>(id: A, f: impl FnOnce(&mut Node) -> B) -> B {
    ARENA.with_borrow(|arena| {
        f(&mut arena[id.borrow().0].borrow_mut())
    })
}


