use super::node::Node;
use super::gc::WeakDom;


/// A NodeIterator is an iterator over nodes, a NodeIterator is cheaply cloned.
#[derive(Clone)]
pub struct NodeIterator {
    prev: Option<WeakDom<Node>>,
    f: fn(&Node) -> Option<WeakDom<Node>>,
}

impl NodeIterator {
    pub fn new(prev: Option<WeakDom<Node>>, f: fn(&Node) -> Option<WeakDom<Node>>) -> NodeIterator {
        NodeIterator {
            prev,
            f,
        }
    }
}

impl Iterator for NodeIterator {
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

/// TreeIterator is an iterator over all tree descendants of a node.
#[derive(Clone)]
pub struct TreeIterator {
    prev: Option<WeakDom<Node>>,
}

impl TreeIterator {
    pub fn new(prev: Option<WeakDom<Node>>) -> TreeIterator {
        TreeIterator {
            prev,
        }
    }
}

impl Iterator for TreeIterator {
    type Item = WeakDom<Node>;

    // TODO: this algorithm only iterates over leaf nodes, we also want it to iterate over internal
    // nodes.
    fn next(&mut self) -> Option<WeakDom<Node>> {
        let prev = self.prev.clone().map(|prev| Node::first_descendant(prev.upgrade()));

        if let Some(node) = &prev && let Some(sibling) = node.borrow().next_sibling.clone() {
            self.prev.replace(WeakDom::new_from_owned(sibling))
        } else if let Some(node) = &prev && let Some(parent) = node.borrow().parent.clone() {
            self.prev.replace(parent)
        } else {
            None
        }
    }
}


