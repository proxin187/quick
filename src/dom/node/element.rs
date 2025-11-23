use crate::dom::node::attribute::Attribute;
use crate::dom::node::{Node, NodeType, QualifiedName};
use crate::dom::arena::NodeId;
use crate::dom::inheritance::{private, Downcast};

use std::rc::Rc;
use std::cell::RefCell;


struct CustomElementRegistry {
    scoped: bool,
}

#[derive(Clone, Default)]
pub struct NullOrCustomElementRegistry {
    registry: Option<Rc<RefCell<CustomElementRegistry>>>,
}

impl NullOrCustomElementRegistry {
    pub fn is_global_custom_element_registry(&self) -> bool {
        self.registry.as_ref()
            .map(|registry| !registry.borrow().scoped).unwrap_or_default()
    }

    pub fn effective_global_custom_element_registry(&self) -> NullOrCustomElementRegistry {
        NullOrCustomElementRegistry {
            registry: self.registry.clone().filter(|registry| !registry.borrow().scoped)
        }
    }
}

pub struct Element {
    pub name: QualifiedName,
    pub custom_element_registry: NullOrCustomElementRegistry,
    pub attributes: Vec<Attribute>,
}

impl private::Sealed for Element {}

impl Downcast<Node> for Element {
    fn downcast_ref(node: &Node) -> &Element {
        match &node.node_type {
            NodeType::Element(element) => element,
            _ => panic!("expected element"),
        }
    }

    fn downcast_mut(node: &mut Node) -> &mut Element {
        match &mut node.node_type {
            NodeType::Element(element) => element,
            _ => panic!("expected element"),
        }
    }
}

impl Element {
}


