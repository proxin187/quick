use crate::dom::node::Node;
use crate::dom::arena::{self, NodeId};


/// A NodeIterator is an iterator over nodes, a NodeIterator is cheaply cloned.
#[derive(Clone)]
pub struct NodeIterator {
    prev: Option<NodeId>,
    f: fn(&Node) -> Option<NodeId>,
}

impl NodeIterator {
    pub fn new(prev: Option<NodeId>, f: fn(&Node) -> Option<NodeId>) -> NodeIterator {
        NodeIterator {
            prev,
            f,
        }
    }
}

impl Iterator for NodeIterator {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        match &self.prev {
            Some(prev) => {
                let next = (self.f)(&arena::get(*prev));

                next.map(|next| self.prev.replace(next)).unwrap_or_else(|| self.prev)
            },
            None => None,
        }
    }
}

/// TreeIterator is an iterator over all tree descendants of a node.
#[derive(Clone)]
pub struct TreeIterator {
    prev: Option<NodeId>,
    depth: usize,
}

// TODO: implement shadow including inclusive descendants
impl TreeIterator {
    pub fn new(prev: Option<NodeId>) -> TreeIterator {
        TreeIterator {
            prev,
            depth: 0,
        }
    }
}

impl Iterator for TreeIterator {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        let prev = self.prev.take()?;

        if let Some(child) = arena::get(prev).first_child {
            self.prev = Some(child);

            self.depth += 1;
        } else {
            for node in NodeIterator::new(self.prev, |node| node.parent) {
                if let Some(sibling) = arena::get(node).next_sibling && self.depth > 0 {
                    self.prev = Some(sibling);

                    break;
                } else if self.depth > 0 {
                    self.depth -= 1;
                }
            }
        }

        Some(prev)
    }
}


