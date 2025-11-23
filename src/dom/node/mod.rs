pub mod document_fragment;
pub mod document;
pub mod element;
pub mod comment;
mod attribute;

use crate::dom::iterators::{NodeIterator, TreeIterator};
use crate::dom::arena::{self, NodeId};

use document_fragment::DocumentFragment;
use document::Document;
use element::Element;
use comment::Comment;


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
    Comment(Comment),
}

pub struct Node {
    /// The type of the node.
    pub(crate) node_type: NodeType,

    /// The owner document of the node.
    pub(crate) node_document: NodeId,

    /// The parent of the node.
    pub(crate) parent: Option<NodeId>,

    /// Previous sibling of the node.
    pub(crate) previous_sibling: Option<NodeId>,

    /// Next sibling of the node.
    pub(crate) next_sibling: Option<NodeId>,

    /// The first child of the node.
    pub(crate) first_child: Option<NodeId>,

    /// The last child of the node.
    pub(crate) last_child: Option<NodeId>,

    /// The count of children of the node.
    pub(crate) child_count: usize,
}

impl Node {
    pub fn new(node_type: NodeType, node_document: NodeId) -> Node {
        Node {
            node_type,
            node_document,
            parent: None,
            previous_sibling: None,
            next_sibling: None,
            first_child: None,
            last_child: None,
            child_count: 0,
        }
    }

    pub fn first_descendant(node: NodeId) -> NodeId {
        arena::get(node).first_child.map(|child| Node::first_descendant(child)).unwrap_or(node)
    }

    pub fn root(node: NodeId) -> NodeId {
        arena::get(node).parent.map(|parent| Node::root(parent)).unwrap_or(node)
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

            // NOTE: previous sibling is only used with the shadow root related steps
            // let previous_sibling = child.map(|node| arena::get(node).previous_sibling)
            //    .unwrap_or_else(|| self.last_child.clone());

            for node in nodes {
                Document::adopt(self.node_document, node);

                if let Some(child) = child {
                    self.insert_before(node, child);
                } else {
                    self.append(node);
                }

                // TODO: implement step 4, 5, 6, and 7 once we have shadow root elements

                // TODO: children changed steps will have to mark the children as dirty when we are
                // to render the next layout tree

                // TODO: implement step 10, 11, and 12 once we have shadow root elements
            }
        }
    }

    fn append(&mut self, node: NodeId) {
        if let Some(first_child) = self.first_child {
            let previous = self.last_child.unwrap_or(first_child);

            arena::with_mut(previous, |previous| previous.next_sibling = Some(node));

            arena::with_mut(node, |node| node.previous_sibling = Some(previous));

            self.last_child = Some(node);
        } else {
            self.first_child = Some(node);
        }

        self.child_count += 1;
    }

    fn insert_before(&mut self, node: NodeId, before: NodeId) {
        if let Some(previous_sibling) = arena::get(before).previous_sibling {
            arena::with_mut(previous_sibling, |previous_sibling| previous_sibling.next_sibling = Some(node));

            arena::with_mut(node, |node| {
                node.previous_sibling = Some(previous_sibling);

                node.next_sibling = Some(before);
            });

            arena::with_mut(before, |before| before.previous_sibling = Some(node));
        } else {
            self.first_child = Some(node);

            arena::with_mut(node, |node| node.next_sibling = Some(before));

            if self.last_child.is_none() {
                self.last_child = Some(before);
            }
        }

        self.child_count += 1;
    }

    fn pre_insert(&mut self, node: Node, child: NodeId) {
    }

    fn remove(&mut self, node: NodeId) {
    }
}


