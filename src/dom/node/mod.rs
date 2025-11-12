mod document_fragment;
mod document;
mod element;

use super::iterators::{NodeIterator, TreeIterator};
use super::gc::{OwnedDom, WeakDom};

use document_fragment::DocumentFragment;
use document::Document;
use element::Element;

use std::rc::Rc;


#[derive(Clone)]
pub enum NodeType {
    Element(OwnedDom<Element>),
    Document(OwnedDom<Document>),
    DocumentFragment(OwnedDom<DocumentFragment>),
}

#[derive(Clone)]
pub struct Node {
    /// The type of the node.
    node_type: NodeType,

    /// The owner document of the node.
    node_document: WeakDom<Document>,

    /// The parent of the node.
    pub(crate) parent: Option<WeakDom<Node>>,

    /// Previous sibling of the node.
    pub(crate) previous_sibling: Option<WeakDom<Node>>,

    /// Next sibling of the node.
    pub(crate) next_sibling: Option<OwnedDom<Node>>,

    /// The first child of the node.
    pub(crate) first_child: Option<OwnedDom<Node>>,

    /// The last child of the node.
    last_child: Option<WeakDom<Node>>,

    /// The count of children of the node.
    child_count: usize,
}

impl Node {
    fn first_descendant(node: OwnedDom<Node>) -> OwnedDom<Node> {
        let first_child = node.borrow().first_child.clone();

        first_child.map(|child| Node::first_descendant(child)).unwrap_or(node)
    }

    pub fn descendants(&self) -> TreeIterator {
        TreeIterator::new(self.first_child.clone().map(|child| WeakDom::new_from_owned(Node::first_descendant(child))))
    }

    pub fn children(&self) -> NodeIterator {
        NodeIterator::new(self.first_child.clone().map(|node| WeakDom::new_from_owned(node)), |node| node.next_sibling.clone().map(|node| WeakDom::new_from_owned(node)))
    }

    pub fn index(&self) -> usize {
        NodeIterator::new(self.previous_sibling.clone(), |node| node.previous_sibling.clone())
            .count()
    }

    fn insert(&mut self, new_node: OwnedDom<Node>, child: Option<WeakDom<Node>>) {
        let nodes = matches!(new_node.borrow().node_type, NodeType::DocumentFragment(_))
            .then(|| new_node.borrow().children().collect::<Vec<WeakDom<Node>>>())
            .unwrap_or_else(|| vec![WeakDom::new_from_owned(Rc::clone(&new_node))]);

        if nodes.len() > 0  {
            if let NodeType::DocumentFragment(_) = new_node.borrow().node_type {
                for node in &nodes {
                    new_node.borrow_mut().remove(node.upgrade());
                }
            }

            if let Some(parent) = &self.parent && let Some(child) = &child {
                for range in self.node_document.upgrade().borrow_mut().ranges.iter_mut() {
                    range.adjust_offset(parent, child, nodes.len());
                }
            }

            let previous_sibling = child.map(|node| node.upgrade().borrow().previous_sibling.clone())
                .unwrap_or_else(|| self.last_child.clone());

            for node in nodes {
                self.node_document.upgrade().borrow().adopt(node.clone());
            }
        }
    }

    fn pre_insert(&mut self, node: Node, child: OwnedDom<Node>) {
    }

    fn remove(&mut self, node: OwnedDom<Node>) {
    }

    pub fn append(&mut self, node: Node) {
    }

    pub fn insert_before(&mut self, node: OwnedDom<Node>, child: Option<Node>) {
    }
}


