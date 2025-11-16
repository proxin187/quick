use crate::dom::node::attribute::Attribute;
use crate::dom::node::{Node, QualifiedName};
use crate::dom::gc::WeakDom;


#[derive(Clone)]
struct CustomElementRegistry {
    scoped: bool,
}

pub struct NullOrCustomElementRegistry {
    registry: Option<CustomElementRegistry>,
}

impl NullOrCustomElementRegistry {
    pub fn is_global_custom_element_registry(&self) -> bool {
        self.registry.as_ref()
            .map(|registry| !registry.scoped).unwrap_or_default()
    }

    pub fn effective_global_custom_element_registry(&self) -> NullOrCustomElementRegistry {
        NullOrCustomElementRegistry {
            registry: self.registry.clone().filter(|registry| !registry.scoped)
        }
    }
}

pub struct Element {
    pub owner: WeakDom<Node>,
    pub name: QualifiedName,
    pub custom_element_registry: NullOrCustomElementRegistry,
    pub attributes: Vec<Attribute>,
}

impl Element {
}


