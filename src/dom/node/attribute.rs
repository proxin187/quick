use crate::dom::node::element::Element;
use crate::dom::node::document::Document;
use crate::dom::node::QualifiedName;
use crate::dom::gc::WeakDom;


/// An attribute node, the spec wants Attribute to extend Node, however it not needed.
pub struct Attribute {
    owner: WeakDom<Element>,
    pub(crate) node_document: WeakDom<Document>,
    pub(crate) name: QualifiedName,
    pub(crate) value: String,
}


