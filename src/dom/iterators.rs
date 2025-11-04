use crate::dom::node::Node;

use std::cell::RefCell;
use std::rc::Rc;


/// A NodeList is an iterator over nodes, a NodeList is cheaply cloned as everything inside is
/// wrapped in Rc.
#[derive(Clone)]
pub struct NodeList {
    next: Option<Rc<RefCell<Node>>>,
    f: fn(&Node) -> Option<Rc<RefCell<Node>>>,
}

impl NodeList {
    pub fn new(next: Option<Rc<RefCell<Node>>>, f: fn(&Node) -> Option<Rc<RefCell<Node>>>) -> NodeList {
        NodeList {
            next,
            f,
        }
    }
}

impl Iterator for NodeList {
    type Item = Rc<RefCell<Node>>;

    fn next(&mut self) -> Option<Rc<RefCell<Node>>> {
        let next = self.next.clone().and_then(|node| (self.f)(&node.borrow()));

        next.map(|next| self.next.replace(next)).unwrap_or_else(|| self.next.clone())
    }
}

// TODO: implement TreeDescendants, first we will have to rework the node system, we will have to
// be able to upcast a node into eg. an element, and downcast that element back down to a node.

/// TreeDescendants is an iterator over all tree descendants of a node.
#[derive(Clone)]
pub struct TreeDescendants {
    next: Option<Rc<RefCell<Node>>>,
    f: fn(&Node) -> Option<Rc<RefCell<Node>>>,
}

impl TreeDescendants {
    pub fn new(next: Option<Rc<RefCell<Node>>>, f: fn(&Node) -> Option<Rc<RefCell<Node>>>) -> TreeDescendants {
        TreeDescendants {
            next,
            f,
        }
    }
}

impl Iterator for TreeDescendants {
    type Item = Rc<RefCell<Node>>;

    fn next(&mut self) -> Option<Rc<RefCell<Node>>> {
        None
    }
}


