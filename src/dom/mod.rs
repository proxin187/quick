mod inheritance;
mod iterators;
mod node;
mod arena;

use arena::NodeId;
use node::{Node, NodeType};
use node::document::Document;
use node::comment::Comment;


// TODO: maybe we should store the arena allocation inside the Dom? if so, how would we do that
// without having to pass around a reference to dom everywhere?
pub struct Dom {
    document: NodeId,
}

impl Dom {
    pub fn new() -> Dom {
        Dom {
            document: arena::insert_cyclic(|document| Node::new(NodeType::Document(Document::default()), document)),
        }
    }

    pub fn create_comment(&mut self, content: String) -> NodeId {
        arena::insert(Node::new(NodeType::Comment(Comment::new(content)), self.document))
    }
}


