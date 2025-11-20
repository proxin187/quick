mod document_fragment;
mod document;
mod attribute;
mod element;

use crate::dom::iterators::{NodeIterator, TreeIterator};
use crate::dom::arena::{self, NodeId};
use crate::dom::inheritance::Downcast;

use document_fragment::DocumentFragment;
use document::Document;
use element::Element;


/// The local name, namespace and namespace prefix of a node in the DOM tree.
#[derive(Clone)]
pub struct QualifiedName {
    pub namespace: Option<String>,
    pub namespace_prefix: Option<String>,
    pub local_name: String,
}

impl QualifiedName {
    /// Create a new qualified name with a local_name, namespace and namespace prefix.
    pub fn new_with_ns(local_name: String, namespace: String, namespace_prefix: Option<String>) -> QualifiedName {
        QualifiedName {
            namespace: Some(namespace),
            namespace_prefix,
            local_name,
        }
    }
}

pub enum NodeType {
    Element(Element),
    Document(Document),
    DocumentFragment(DocumentFragment),
}

pub struct Node {
    /// The type of the node.
    node_type: NodeType,

    /// The owner document of the node.
    node_document: NodeId,

    /// The parent of the node.
    pub(crate) parent: Option<NodeId>,

    /// Previous sibling of the node.
    pub(crate) previous_sibling: Option<NodeId>,

    /// Next sibling of the node.
    pub(crate) next_sibling: Option<NodeId>,

    /// The first child of the node.
    pub(crate) first_child: Option<NodeId>,

    /// The last child of the node.
    last_child: Option<NodeId>,

    /// The count of children of the node.
    child_count: usize,
}

impl Node {
    fn first_descendant(node: NodeId) -> NodeId {
        let first_child = arena::get(node).first_child;

        first_child.map(|child| Node::first_descendant(child)).unwrap_or(node)
    }

    pub fn descendants(&self) -> TreeIterator {
        TreeIterator::new(self.first_child.map(|child| Node::first_descendant(child)))
    }

    pub fn children(&self) -> NodeIterator {
        NodeIterator::new(self.first_child, |node| node.next_sibling)
    }

    pub fn index(&self) -> usize {
        NodeIterator::new(self.previous_sibling, |node| node.previous_sibling).count()
    }

    #[inline]
    pub fn downcast_ref<T: Downcast<Node>>(&self) -> &T {
        T::downcast_ref(self)
    }

    #[inline]
    pub fn downcast_mut<T: Downcast<Node>>(&mut self) -> &mut T {
        T::downcast_mut(self)
    }

    // TODO: finish insert
    pub fn insert(&mut self, new_node: NodeId, child: Option<NodeId>) {
        let nodes = matches!(arena::get(new_node).node_type, NodeType::DocumentFragment(_))
            .then(|| arena::get(new_node).children().collect::<Vec<NodeId>>())
            .unwrap_or_else(|| vec![new_node]);

        if nodes.len() > 0  {
            arena::with_mut(new_node, |new_node| {
                if let NodeType::DocumentFragment(_) = new_node.node_type {
                    for node in &nodes {
                        new_node.remove(*node);
                    }
                }
            });

            if let Some(parent) = self.parent && let Some(child) = child {
                arena::with_mut(self.node_document, |node_document| {
                    for range in node_document.downcast_mut::<Document>().ranges.iter_mut() {
                        range.adjust_offset(parent, child, nodes.len());
                    }
                });
            }

            let previous_sibling = child.map(|node| arena::get(node).previous_sibling)
                .unwrap_or_else(|| self.last_child.clone());

            for node in nodes {
                Document::adopt(self.node_document, node);
            }
        }
    }

    fn pre_insert(&mut self, node: Node, child: NodeId) {
    }

    fn remove(&mut self, node: NodeId) {
    }

    pub fn append(&mut self, node: Node) {
    }

    pub fn insert_before(&mut self, node: NodeId, child: Option<Node>) {
    }
}


