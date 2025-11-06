use crate::dom::node::Node;

use std::cell::RefCell;
use std::rc::Rc;


/// A NodeList is an iterator over nodes, a NodeList is cheaply cloned.
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

/// TreeDescendants is an iterator over all tree descendants of a node.
#[derive(Clone)]
pub struct TreeDescendants {
    nodes: NodeList,
}

impl TreeDescendants {
    pub fn new(parent: Rc<RefCell<Node>>) -> TreeDescendants {
        TreeDescendants {
            nodes: parent.borrow().children(),
        }
    }
}

impl Iterator for TreeDescendants {
    type Item = Rc<RefCell<Node>>;

    fn next(&mut self) -> Option<Rc<RefCell<Node>>> {
        self.nodes.flat_map(||)

        None
    }
}


