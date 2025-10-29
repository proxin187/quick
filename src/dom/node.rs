use super::document_fragment::DocumentFragment;
use super::document::Document;
use super::element::Element;

use std::cell::RefCell;
use std::rc::Rc;


/// A NodeList is an iterator over nodes.
#[derive(Clone)]
pub struct NodeList {
    next: Option<Rc<RefCell<Node>>>,
}

impl NodeList {
    pub fn new(next: Option<Rc<RefCell<Node>>>) -> NodeList {
        NodeList {
            next,
        }
    }
}

impl Iterator for NodeList {
    type Item = Rc<RefCell<Node>>;

    fn next(&mut self) -> Option<Rc<RefCell<Node>>> {
        self.next = self.next.take().and_then(|node| node.borrow().next_sibling.clone());

        self.next.clone()
    }
}

#[derive(Clone)]
pub enum NodeType {
    Element(Element),
    Document(Document),
    DocumentFragment(DocumentFragment),
}

#[derive(Clone)]
pub struct Node {
    /// The type of the node.
    node_type: NodeType,

    /// The owner document of the node.
    owner_document: Rc<RefCell<Document>>,

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

impl Node {
    fn children(&self) -> NodeList {
        NodeList::new(self.first_child.clone())
    }

    fn insert(&mut self, node: Rc<RefCell<Node>>, child: Option<Rc<RefCell<Node>>>) {
        let (nodes, count) = matches!(node.borrow().node_type, NodeType::DocumentFragment(_))
            .then(|| (Box::new(node.borrow().children()) as Box<dyn Iterator<Item = Rc<RefCell<Node>>>>, node.borrow().child_count))
            .unwrap_or_else(|| (Box::new(vec![Rc::clone(&node)].into_iter()) as Box<dyn Iterator<Item = Rc<RefCell<Node>>>>, 1));

        if count > 0  {
            if let NodeType::DocumentFragment(_) = node.borrow().node_type {
                for child in nodes {
                    node.borrow_mut().remove(Rc::clone(&child));
                }
            }

            let previous_sibling = child.map(|node| node.borrow().previous_sibling.clone())
                .unwrap_or_else(|| self.last_child.clone());
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


