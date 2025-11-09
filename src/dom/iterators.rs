use super::node::Node;
use super::gc::WeakDom;

use std::cell::RefCell;
use std::rc::Rc;


/// A NodeList is an iterator over nodes, a NodeList is cheaply cloned.
#[derive(Clone)]
pub struct NodeList {
    prev: Option<WeakDom<Node>>,
    f: fn(&Node) -> Option<WeakDom<Node>>,
}

impl NodeList {
    pub fn new(prev: Option<WeakDom<Node>>, f: fn(&Node) -> Option<WeakDom<Node>>) -> NodeList {
        NodeList {
            prev,
            f,
        }
    }
}

impl Iterator for NodeList {
    type Item = WeakDom<Node>;

    fn next(&mut self) -> Option<WeakDom<Node>> {
        match &self.prev {
            Some(prev) => {
                let next = (self.f)(&prev.upgrade().borrow());

                next.map(|next| self.prev.replace(next)).unwrap_or_else(|| self.prev.clone())
            },
            None => None,
        }
    }
}

/// TreeDescendants is an iterator over all tree descendants of a node.
#[derive(Clone)]
pub struct TreeDescendants {
    nodes: NodeList,
}

impl TreeDescendants {
    pub fn new(parent: WeakDom<Node>) -> TreeDescendants {
        TreeDescendants {
            nodes: parent.upgrade()
                .borrow()
                .children(),
        }
    }
}

impl Iterator for TreeDescendants {
    type Item = Rc<RefCell<Node>>;

    fn next(&mut self) -> Option<Rc<RefCell<Node>>> {
        /*
        self.nodes.flat_map(||)

        None
        */

        todo!()
    }
}


