mod document_fragment;
mod document;
mod element;

use super::iterators::NodeList;
use super::inheritance::{Castable, UpcastRef};

use document_fragment::DocumentFragment;
use document::Document;
use element::Element;

use std::cell::RefCell;
use std::rc::Rc;
use std::any::TypeId;


#[derive(Clone)]
pub enum NodeType {
    Element(Rc<RefCell<Element>>),
    Document(Rc<RefCell<Document>>),
    DocumentFragment(Rc<RefCell<DocumentFragment>>),
}

#[derive(Clone)]
pub struct Node {
    /// The type of the node.
    node_type: NodeType,

    /// The owner document of the node.
    node_document: Rc<RefCell<Document>>,

    /// The parent of the node.
    parent: Option<Rc<RefCell<Node>>>,

    /// Previous sibling of the node.
    previous_sibling: Option<Rc<RefCell<Node>>>,

    /// Next sibling of the node.
    next_sibling: Option<Rc<RefCell<Node>>>,

    /// The first child of the node.
    first_child: Option<Rc<RefCell<Node>>>,

    /// The last child of the node.
    last_child: Option<Rc<RefCell<Node>>>,

    /// The count of children of the node.
    child_count: usize,
}

impl Castable for Node {
    fn upcast<T: 'static>(&self) -> Option<UpcastRef<T>> {
        todo!()
    }
}

impl Node {
    pub fn children(&self) -> NodeList {
        NodeList::new(self.first_child.clone(), |node| node.next_sibling.clone())
    }

    pub fn index(&self) -> usize {
        NodeList::new(self.previous_sibling.clone(), |node| node.previous_sibling.clone())
            .count()
    }

    fn insert(&mut self, new_node: Rc<RefCell<Node>>, child: Option<Rc<RefCell<Node>>>) {
        let nodes = matches!(new_node.borrow().node_type, NodeType::DocumentFragment(_))
            .then(|| new_node.borrow().children().collect::<Vec<Rc<RefCell<Node>>>>())
            .unwrap_or_else(|| vec![new_node.clone()]);

        if nodes.len() > 0  {
            if let NodeType::DocumentFragment(_) = new_node.borrow().node_type {
                for node in &nodes {
                    node.borrow_mut().remove(Rc::clone(&node));
                }
            }

            if let Some(parent) = &self.parent && let Some(child) = &child {
                for range in self.node_document.borrow_mut().ranges.iter_mut() {
                    range.adjust_offset(parent, child, nodes.len());
                }
            }

            let previous_sibling = child.map(|node| node.borrow().previous_sibling.clone())
                .unwrap_or_else(|| self.last_child.clone());

            for node in nodes {
                self.node_document.borrow().adopt(node.clone());
            }
        }
    }

    fn pre_insert(&mut self, node: Node, child: Rc<RefCell<Node>>) {
    }

    fn remove(&mut self, node: Rc<RefCell<Node>>) {
    }

    pub fn append(&mut self, node: Node) {
    }

    pub fn insert_before(&mut self, node: Rc<RefCell<Node>>, child: Option<Node>) {
    }
}


