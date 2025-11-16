use crate::dom::gc::WeakDom;
use crate::dom::iterators::TreeIterator;
use crate::dom::node::{Node, NodeType};
use crate::dom::node::element::NullOrCustomElementRegistry;

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
    pub custom_element_registry: NullOrCustomElementRegistry,
    pub ranges: Vec<Range>,
}

impl Document {
    pub fn adopt(document: WeakDom<Document>, node: WeakDom<Node>) {
        if let Some(parent) = &node.upgrade().borrow().parent {
            parent.upgrade().borrow_mut().remove(node.upgrade());
        }

        if !Weak::ptr_eq(&node.upgrade().borrow().node_document.upgrade().borrow().owner.inner, &document.upgrade().borrow().owner.inner) {
            for descendant in TreeIterator::new(Some(WeakDom::clone(&node))).map(|weak| weak.upgrade()) {
                descendant.borrow_mut().node_document = WeakDom::clone(&document);

                // TODO: step 2: shadow root thing

                if let NodeType::Element(element) = &descendant.borrow().node_type {
                    for attribute in element.borrow_mut().attributes.iter_mut() {
                        attribute.node_document = WeakDom::clone(&document);
                    }

                    if element.borrow().custom_element_registry.is_global_custom_element_registry() {
                        // TODO: figure out whether we should return a clone or a reference.
                        // The name "global custom element registry" suggests we might have to return a
                        // reference for it to be global across all elements that share it
                        element.borrow_mut().custom_element_registry = document.upgrade().borrow().custom_element_registry.effective_global_custom_element_registry();
                    }
                }
            }

            // TODO: custom element callback reaction

            for descendant in TreeIterator::new(Some(node)).map(|weak| weak.upgrade()) {
            }
        }
    }
}


