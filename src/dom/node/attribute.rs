use crate::dom::node::QualifiedName;
use crate::dom::arena::NodeId;


/// An attribute node, the spec wants Attribute to extend Node, however its not needed.
pub struct Attribute {
    pub(crate) node_document: NodeId,
    pub(crate) name: QualifiedName,
    pub(crate) value: String,
}


