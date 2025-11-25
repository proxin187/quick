mod inheritance;
mod iterators;
mod node;
mod arena;

use arena::{Arena, NodeId};
use node::{Node, NodeType};
use node::document::Document;
use node::comment::Comment;


pub struct Dom {
    document: NodeId,
    arena: Arena,
}

impl Dom {
    pub fn new() -> Dom {
        Dom {
            document: arena::insert_cyclic(|document| Node::new(NodeType::Document(Document::default()), document)),
            arena: Arena::new(),
        }
    }

    pub fn create_comment(&mut self, content: String) -> NodeId {
        arena::insert(Node::new(NodeType::Comment(Comment::new(content)), self.document))
    }
}


