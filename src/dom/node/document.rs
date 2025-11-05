use super::Node;

use std::cell::RefCell;
use std::rc::Rc;


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

pub struct Document {
    pub owner: Rc<RefCell<Node>>,
    pub ranges: Vec<Range>,
}

impl Document {
    pub fn adopt(&self, node: Rc<RefCell<Node>>) {
        if let Some(parent) = &node.borrow().parent {
            parent.borrow_mut().remove(node.clone());
        }

        if !Rc::ptr_eq(&node.borrow().node_document.borrow().owner, &self.owner) {
        }
    }
}


