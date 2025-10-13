pub mod interface;
mod state;

use crate::tokenizer::{TokenSink, Token, Tag, TagKind};

use state::InsertionMode;
use interface::TreeSink;


pub struct TreeBuilder<Handle, Sink: TreeSink<Handle>> {
    sink: Sink,
    insertion_mode: InsertionMode,
    open_elements: Vec<Handle>,
}

impl<Handle, Sink: TreeSink<Handle>> TreeBuilder<Handle, Sink> {
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

    // NOTE: currently this assumes its not a html fragment parser.
    fn adjusted_current_node(&self) -> &Handle {
        self.current_node()
    }

    fn not_foreign(&self, token: &Token) -> bool {
        let element_name = self.sink.element_name(self.adjusted_current_node());

        self.open_elements.is_empty()
            || element_name.is_namespace("http://www.w3.org/1999/xhtml")
            || (element_name.is_mathml_text_integration_point() && !(token.is_start_tag("mglyph") || token.is_start_tag("malignmark")))
            || (element_name.is_mathml_text_integration_point() && matches!(token, Token::Character(_)))
            || (element_name.is_mathml_annotation_xml() && token.is_start_tag("svg"))
            || (element_name.is_html_integration_point() && matches!(token, Token::Tag(Tag { kind: TagKind::Start, .. }) | Token::Character(_)))
    }
}

impl<Handle, Sink: TreeSink<Handle>> TokenSink for TreeBuilder<Handle, Sink> {
    fn process(&mut self, token: Token) {
        if self.not_foreign(&token) {
        } else {
        }
    }

    fn eof(&mut self) {
        // TODO: end of file tokens are handled as not foreign
    }
}


