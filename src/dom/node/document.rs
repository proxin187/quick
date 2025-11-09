use super::Node;

use crate::dom::gc::WeakDom;

use std::rc::{Rc, Weak};


pub struct Boundary {
    container: WeakDom<Node>,
    offset: usize,
}

impl Boundary {
    pub fn new(container: WeakDom<Node>, offset: usize) -> Boundary {
        Boundary {
            container,
            offset,
        }
    }
}

pub struct Range {
    start: Boundary,
    end: Boundary,
}

impl Range {
    pub fn new(start: Boundary, end: Boundary) -> Range {
        Range {
            start,
            end,
        }
    }

    pub fn adjust_offset(&mut self, parent: &WeakDom<Node>, child: &WeakDom<Node>, count: usize) {
        if Weak::ptr_eq(&self.start.container.inner, &parent.inner) && self.start.offset > child.upgrade().borrow().index() {
            self.start.offset += count;
        }

        if Weak::ptr_eq(&self.end.container.inner, &parent.inner) && self.end.offset > child.upgrade().borrow().index() {
            self.end.offset += count;
        }
    }
}

pub struct Document {
    pub owner: WeakDom<Node>,
    pub ranges: Vec<Range>,
}

impl Document {
    pub fn adopt(&self, weak_node: WeakDom<Node>) {
        let node = weak_node.upgrade();

        if let Some(parent) = &Rc::clone(&node).borrow().parent {
            parent.upgrade().borrow_mut().remove(Rc::clone(&node));
        }

        if !Weak::ptr_eq(&node.borrow().node_document.upgrade().borrow().owner.inner, &self.owner.inner) {
            // TODO: finish adopt
        }
    }
}


