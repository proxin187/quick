use crate::parser::interface::QualifiedName;

use crate::dom::node::Node;
use crate::dom::gc::WeakDom;

// TODO: move QualifiedName into dom and remove lifetime

// NOTE: here we intentionally ignore node document and element because they arent needed for our
// implementation.
pub struct Attribute {
    qualified_name: QualifiedName<'static>,
    value: String,
}

pub struct Element {
    owner: WeakDom<Node>,
    attributes: Vec<Attribute>,
}


