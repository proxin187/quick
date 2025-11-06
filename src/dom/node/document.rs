use super::Node;

use std::cell::RefCell;
use std::rc::{Rc, Weak};


pub struct Boundary {
    container: Rc<RefCell<Node>>,
    offset: usize,
}

impl Boundary {
    pub fn new(container: Rc<RefCell<Node>>, offset: usize) -> Boundary {
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

    pub fn adjust_offset(&mut self, parent: &Rc<RefCell<Node>>, child: &Rc<RefCell<Node>>, count: usize) {
        if Rc::ptr_eq(&self.start.container, parent) && self.start.offset > child.borrow().index() {
            self.start.offset += count;
        }

        if Rc::ptr_eq(&self.end.container, parent) && self.end.offset > child.borrow().index() {
            self.end.offset += count;
        }
    }
}

// TODO: we will have to make a wrapper around Rc and Weak so that we dont have to call
// upgrade().expect("node dropped") every single time we want to have a weak pointer.

pub struct Document {
    pub owner: Weak<RefCell<Node>>,
    pub ranges: Vec<Range>,
}

impl Document {
    pub fn adopt(&self, node: Rc<RefCell<Node>>) {
        if let Some(parent) = &node.borrow().parent.clone().and_then(|parent| parent.upgrade()) {
            parent.borrow_mut().remove(node.clone());
        }

        let document = node.borrow().node_document.upgrade().expect("node document dropped").borrow().owner.upgrade().expect("node dropped");
        let old_document = self.owner.upgrade().expect("node dropped");

        if !Rc::ptr_eq(&document, &old_document) {
        }
    }
}


