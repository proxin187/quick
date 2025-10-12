pub mod interface;
mod state;

use crate::tokenizer::{TokenSink, Token};

use state::InsertionMode;
use interface::TreeSink;


pub struct TreeBuilder<Handle, Sink: TreeSink> {
    sink: Sink,
    insertion_mode: InsertionMode,
    open_elements: Vec<Handle>,
}

impl<Handle, Sink: TreeSink> TreeBuilder<Handle, Sink> {
    pub fn new(sink: Sink) -> TreeBuilder<Handle, Sink> {
        TreeBuilder {
            sink,
            insertion_mode: InsertionMode::Initial,
            open_elements: Vec::new(),
        }
    }

    fn current_node(&self) -> &Handle {
        self.open_elements.last().expect("no current node")
    }

    // TODO: currently this assumes its not a html fragment parser.
    fn adjusted_current_node(&self) -> &Handle {
        self.current_node()
    }

    // TODO: we need to figure out html namespaces in order to make this function work properly
    fn not_foreign(&self, token: &Token) -> bool {
        self.open_elements.is_empty()
    }
}

impl<Handle, Sink: TreeSink> TokenSink for TreeBuilder<Handle, Sink> {
    fn process(&mut self, token: Token) {
    }

    fn eof(&mut self) {
    }
}


