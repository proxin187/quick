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
    depth: usize,
}

impl TreeIterator {
    pub fn new(prev: Option<WeakDom<Node>>) -> TreeIterator {
        TreeIterator {
            prev,
            depth: 0,
        }
    }
}

impl<'a> Iterator for TreeIterator {
    type Item = WeakDom<Node>;

    fn next(&mut self) -> Option<WeakDom<Node>> {
        let prev = self.prev.take()?;

        if let Some(child) = prev.upgrade().borrow().first_child.clone() {
            self.prev = Some(WeakDom::new_from_owned(child));

            self.depth += 1;
        } else {
            for node in NodeIterator::new(self.prev.clone(), |node| node.parent.clone()) {
                if let Some(sibling) = node.upgrade().borrow().next_sibling.clone() && self.depth > 0 {
                    self.prev = Some(WeakDom::new_from_owned(sibling));

                    break;
                } else if self.depth > 0 {
                    self.depth -= 1;
                }
            }
        }

        Some(prev)
    }
}


