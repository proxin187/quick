use super::document_fragment::DocumentFragment;
use super::document::Document;
use super::element::Element;

use std::cell::RefCell;
use std::rc::Rc;


pub enum NodeType {
    Element(Element),
    Document(Document),
    DocumentFragment(DocumentFragment),
}

pub struct Node {
    pub type_: NodeType,
    pub is_connected: bool,
    pub owner_document: Rc<RefCell<Document>>,
    pub parent: Option<Rc<RefCell<Node>>>,
    pub children: Vec<Rc<RefCell<Node>>>,
}

impl Node {
    // TODO: this function makes me puke, web specs are really not designed to look good when
    // implemented in rust....
    fn insert(&mut self, node: Node, child: Option<Rc<RefCell<Node>>>) {
        let is_document_fragment = matches!(node.type_, NodeType::DocumentFragment(_));

        let nodes = if is_document_fragment {
            for child in &node.children {
            }

            node.children
        } else {
            vec![Rc::new(RefCell::new(node))]
        };

        if nodes.len() > 0 {
        }
    }

    fn pre_insert(&mut self, node: Node, child: Rc<RefCell<Node>>) {
    }

    pub fn append(&mut self, node: Node) {
        self.children.push(Rc::new(RefCell::new(node)));
    }

    pub fn insert_before(&mut self, node: Rc<RefCell<Node>>, child: Option<Node>) {
    }
}


