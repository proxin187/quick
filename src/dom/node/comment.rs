use crate::dom::inheritance::{private, Downcast};
use crate::dom::node::{Node, NodeType};


pub struct Comment {
    data: String,
}

impl private::Sealed for Comment {}

impl Downcast<Node> for Comment {
    fn downcast_ref(node: &Node) -> &Comment {
        match &node.node_type {
            NodeType::Comment(comment) => comment,
            _ => panic!("expected comment"),
        }
    }

    fn downcast_mut(node: &mut Node) -> &mut Comment {
        match &mut node.node_type {
            NodeType::Comment(comment) => comment,
            _ => panic!("expected comment"),
        }
    }
}

impl Comment {
    pub fn new(data: String) -> Comment {
        Comment {
            data,
        }
    }
}


