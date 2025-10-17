pub mod interface;
pub mod quirks;
mod state;

use crate::tokenizer::{TokenSink, Token, Tag, TagKind};

use state::InsertionMode;
use interface::{TreeSink, ElementName};
use quirks::QuirksMode;


pub struct TreeBuilder<Handle, Sink: TreeSink<Handle>> {
    sink: Sink,
    mode: InsertionMode,
    open_elements: Vec<Handle>,
}

impl<Handle, Sink: TreeSink<Handle>> TreeBuilder<Handle, Sink> {
    pub fn new(sink: Sink) -> TreeBuilder<Handle, Sink> {
        TreeBuilder {
            sink,
            mode: InsertionMode::Initial,
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

    fn step(&mut self, token: Token) {
        let document = self.sink.document();

        match self.mode {
            InsertionMode::Initial => match token {
                Token::Character('\u{0009}' | '\u{000a}' | '\u{000c}' | '\u{000d}' | ' ') => {},
                Token::Comment(content) => {
                    let comment = self.sink.create_comment(content);

                    self.sink.append(&document, &comment);
                },
                Token::Doctype(doctype) => {
                    if doctype.is_parse_error() {
                        self.sink.parse_error("bad doctype");
                    }

                    self.sink.append_doctype(&doctype);

                    self.sink.set_quirks_mode(QuirksMode::from(doctype));

                    self.mode = InsertionMode::BeforeHtml;
                },
                _ => {
                    self.sink.parse_error("not an iframe srcdoc");

                    self.sink.set_quirks_mode(QuirksMode::Quirks);

                    self.mode = InsertionMode::BeforeHtml;

                    self.step(token);
                },
            },
            InsertionMode::BeforeHtml => match token {
                Token::Character('\u{0009}' | '\u{000a}' | '\u{000c}' | '\u{000d}' | ' ') => {},
                Token::Doctype(doctype) => self.sink.parse_error(format!("unexpected: {:?}", doctype)),
                Token::Comment(content) => {
                    let comment = self.sink.create_comment(content);

                    self.sink.append(&document, &comment);
                },
                Token::Tag(tag) if tag.kind == TagKind::Start && tag.name.as_str() == "html" => {
                    let name = ElementName::new(Some("http://www.w3.org/1999/xhtml"), None, tag.name.as_str());

                    let element = self.sink.create_element(name, &tag.attributes);
                    let document = self.sink.document();

                    self.sink.append(&document, &element);

                    self.open_elements.push(element);
                },
                Token::Tag(tag) if tag.kind == TagKind::End && !["head", "body", "html", "br"].contains(&tag.name.as_str()) => {
                    self.sink.parse_error(format!("unexpected: {:?}", tag));
                },
                _ => {
                    // TODO: anything else field
                },
            },
            _ => todo!(),
        }
    }
}

impl<Handle, Sink: TreeSink<Handle>> TokenSink for TreeBuilder<Handle, Sink> {
    fn process(&mut self, token: Token) {
        if self.not_foreign(&token) {
            self.step(token);
        } else {
        }
    }

    fn eof(&mut self) {
        // TODO: end of file tokens are handled as not foreign
    }
}


