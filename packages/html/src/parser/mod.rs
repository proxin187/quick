pub mod interface;
pub mod quirks;
mod state;

use crate::tokenizer::{TokenSink, Token, Tag, TagKind};

use state::InsertionMode;
use interface::{TreeSink, ElementName};
use quirks::QuirksMode;


enum InsertionPoint<'a, Handle> {
    LastChild(&'a Handle),
    BeforeChild(&'a Handle, &'a Handle),
}

pub struct TreeBuilder<Handle, Sink: TreeSink<Handle>> {
    sink: Sink,
    mode: InsertionMode,
    document: Handle,
    open_elements: Vec<Handle>,
    foster_parenting: bool,
}

impl<Handle, Sink: TreeSink<Handle>> TreeBuilder<Handle, Sink> {
    pub fn new(sink: Sink) -> TreeBuilder<Handle, Sink> {
        let document = sink.document();

        TreeBuilder {
            sink,
            mode: InsertionMode::Initial,
            document,
            open_elements: Vec::new(),
            foster_parenting: false,
        }
    }

    fn current_node(&self) -> &Handle {
        self.open_elements.last().expect("no current node")
    }

    // NOTE: currently this assumes its not a html fragment parser.
    fn adjusted_current_node(&self) -> &Handle {
        self.current_node()
    }

    fn last_open_element(&self, local_name: &str) -> (usize, Option<&Handle>) {
        let index = self.open_elements.iter()
            .enumerate()
            .rev()
            .find_map(|(index, handle)| (self.sink.element_name(&handle).local_name == local_name).then(|| index));

        let element = self.open_elements.iter()
            .filter(|handle| self.sink.element_name(&handle).local_name == local_name)
            .last();

        (index.unwrap_or_default(), element)
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

    fn adjusted_insertion_location<'a>(&'a self, target: &'a Handle) -> InsertionPoint<'a, Handle> {
        let name = self.sink.element_name(target);

        if self.foster_parenting && ["table", "tbody", "tfoot", "thead", "tr"].contains(&name.local_name) {
            let (template_index, template) = self.last_open_element("template");
            let (table_index, table) = self.last_open_element("table");

            if let Some(handle) = template && table_index < template_index {
                InsertionPoint::LastChild(handle)
            } else if table.is_none() {
                InsertionPoint::LastChild(&self.open_elements[0])
            } else if let Some(handle) = table && let Some(parent) = self.sink.parent_of(&handle) {
                InsertionPoint::BeforeChild(handle, parent)
            } else {
                InsertionPoint::LastChild(&self.open_elements[table_index - 1])
            }
        } else {
            InsertionPoint::LastChild(target)
        }
    }

    fn appropriate_insertion_point<'a>(&'a self, override_: Option<&'a Handle>) -> InsertionPoint<'a, Handle> {
        let target = override_.unwrap_or_else(|| self.current_node());

        self.adjusted_insertion_location(&target)
    }

    // NOTE: ladybird follows the spec here, however i noticed that servo's html5ever doesnt seem
    // to propeply follow the spec here, maybe we dont have to either?
    fn create_element_for(&self, tag: &Tag) {
    }

    fn insert_foreign_element(&mut self) {
        let adjusted_insertion_location = self.appropriate_insertion_point(None);
    }

    fn append_comment(&mut self, content: &str) {
        let comment = self.sink.create_comment(content);

        self.sink.append(&self.document, &comment);
    }

    #[inline]
    fn reprocess(&mut self, token: Token, mode: InsertionMode) {
        self.mode = mode;

        self.step(token);
    }

    fn step(&mut self, token: Token) {
        match self.mode {
            InsertionMode::Initial => match token {
                Token::Character('\u{0009}' | '\u{000a}' | '\u{000c}' | '\u{000d}' | ' ') => {},
                Token::Comment(content) => self.append_comment(content),
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

                    self.reprocess(token, InsertionMode::BeforeHtml);
                },
            },
            InsertionMode::BeforeHtml => match token {
                Token::Character('\u{0009}' | '\u{000a}' | '\u{000c}' | '\u{000d}' | ' ') => {},
                Token::Doctype(doctype) => self.sink.parse_error(format!("unexpected: {:?}", doctype)),
                Token::Comment(content) => self.append_comment(content),
                Token::Tag(tag) if tag.kind == TagKind::Start && tag.name.as_str() == "html" => {
                    let name = ElementName::new_with_ns(tag.name.as_str(), "http://www.w3.org/1999/xhtml");

                    let element = self.sink.create_element(name, &tag.attributes);

                    self.sink.append(&self.document, &element);

                    self.open_elements.push(element);
                },
                Token::Tag(tag) if tag.kind == TagKind::End && !["head", "body", "html", "br"].contains(&tag.name.as_str()) => {
                    self.sink.parse_error(format!("unexpected: {:?}", tag));
                },
                _ => {
                    let element = self.sink.create_element(ElementName::new_with_ns("html", "http://www.w3.org/1999/xhtml"), &[]);

                    self.sink.append(&self.document, &element);

                    self.open_elements.push(element);

                    self.reprocess(token, InsertionMode::BeforeHead);
                },
            },
            InsertionMode::BeforeHead => match token {
                Token::Character('\u{0009}' | '\u{000a}' | '\u{000c}' | '\u{000d}' | ' ') => {},
                Token::Comment(content) => self.append_comment(content),
                Token::Doctype(doctype) => self.sink.parse_error(format!("unexpected: {:?}", doctype)),
                Token::Tag(tag) if tag.kind == TagKind::Start && tag.name.as_str() == "html" => {
                    self.reprocess(token, InsertionMode::InBody);

                    self.mode = InsertionMode::BeforeHead;
                },
                Token::Tag(tag) if tag.kind == TagKind::Start && tag.name.as_str() == "head" => {
                },
                _ => {
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


