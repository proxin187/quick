use crate::dom::arena::{self, NodeId};
use crate::dom::iterators::TreeIterator;
use crate::dom::node::{Node, NodeType};
use crate::dom::node::element::NullOrCustomElementRegistry;
use crate::dom::inheritance::{private, Downcast};


pub struct Boundary {
    container: NodeId,
    offset: usize,
}

impl Boundary {
    pub fn new(container: NodeId, offset: usize) -> Boundary {
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

    pub fn adjust_offset(&mut self, parent: NodeId, child: NodeId, count: usize) {
        if self.start.container == parent && self.start.offset > arena::get(child).index() {
            self.start.offset += count;
        }

        if self.end.container == parent && self.end.offset > arena::get(child).index() {
            self.end.offset += count;
        }
    }
}

pub struct Document {
    pub custom_element_registry: NullOrCustomElementRegistry,
    pub ranges: Vec<Range>,
}

impl private::Sealed for Document {}

impl Downcast<Node> for Document {
    fn downcast_ref(node: &Node) -> &Document {
        match &node.node_type {
            NodeType::Document(document) => document,
            _ => panic!("expected document"),
        }
    }

    fn downcast_mut(node: &mut Node) -> &mut Document {
        match &mut node.node_type {
            NodeType::Document(document) => document,
            _ => panic!("expected document"),
        }
    }
}

impl Document {
    pub fn adopt(document: NodeId, node: NodeId) {
        if let Some(parent) = arena::get(node).parent {
            arena::with_mut(parent, |parent| parent.remove(node));
        }

        if arena::get(node).node_document != document {
            for descendant in TreeIterator::new(Some(node)) {
                arena::with_mut(descendant, |descendant| {
                    descendant.node_document = document;

                    // TODO: step 2: shadow root thing

                    if let NodeType::Element(element) = &mut descendant.node_type {
                        for attribute in element.attributes.iter_mut() {
                            attribute.node_document = document;
                        }

                        if element.custom_element_registry.is_global_custom_element_registry() {
                            let registry = arena::get(document).downcast_ref::<Document>().custom_element_registry.clone();

                            // TODO: figure out whether we should return a clone or a reference.
                            // The name "global custom element registry" suggests we might have to return a
                            // reference for it to be global across all elements that share it
                            element.custom_element_registry = registry.effective_global_custom_element_registry();
                        }
                    }
                });
            }

            // TODO: custom element callback reaction

            for descendant in TreeIterator::new(Some(node)) {
            }
        }
    }
}


