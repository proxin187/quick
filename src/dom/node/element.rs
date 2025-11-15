use crate::dom::node::attribute::Attribute;
use crate::dom::node::{Node, QualifiedName};
use crate::dom::gc::WeakDom;


pub struct CustomElementRegistry {
    scoped: bool,
}

pub struct Element {
    pub owner: WeakDom<Node>,
    pub name: QualifiedName,
    pub custom_element_registry: Option<CustomElementRegistry>,
    pub attributes: Vec<Attribute>,
}

impl Element {
    pub fn is_global_custom_element_registry(&self) -> bool {
        self.custom_element_registry.as_ref().map(|registry| !registry.scoped).unwrap_or_default()
    }
}


