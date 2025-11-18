use crate::dom::node::Node;

use std::cell::{RefCell, Ref};


thread_local! {
    static ARENA: RefCell<Vec<RefCell<Node>>> = RefCell::new(Vec::new());
}

/// NodeId is an index to an element inside the arena.
#[derive(Clone, Copy, PartialEq)]
pub struct NodeId(usize);

/// Insert a node into the arena.
#[inline]
pub fn insert(node: Node) -> NodeId {
    ARENA.with_borrow_mut(|arena| {
        arena.push(RefCell::new(node));

        NodeId(arena.len() - 1)
    })
}

// TODO: double check that this is actually safe
/// Get a node from the arena.
#[inline]
pub fn get<'a>(id: NodeId) -> Ref<'a, Node> {
    ARENA.with_borrow(|arena| unsafe { std::mem::transmute(arena[id.0].borrow()) })
}


