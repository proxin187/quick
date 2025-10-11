pub mod interface;
mod state;

use crate::tokenizer::{TokenSink, Token};

use state::InsertionMode;
use interface::TreeSink;


pub struct TreeBuilder<Sink: TreeSink> {
    sink: Sink,
    insertion_mode: InsertionMode,
}

impl<Sink: TreeSink> TreeBuilder<Sink> {
    pub fn new(sink: Sink) -> TreeBuilder<Sink> {
        TreeBuilder {
            sink,
            insertion_mode: InsertionMode::Initial,
        }
    }
}

impl<Sink: TreeSink> TokenSink for TreeBuilder<Sink> {
    fn process(&mut self, token: Token) {
    }

    fn eof(&mut self) {
    }
}


