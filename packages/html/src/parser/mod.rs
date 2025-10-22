pub mod interface;
pub mod quirks;
mod state;

use crate::tokenizer::{TokenSink, Token, Tag, TagKind};

use state::InsertionMode;
use interface::{TreeSink, Node, QualifiedName};
use quirks::QuirksMode;


enum InsertionPoint<Handle> {
    LastChild(Handle),
    BeforeChild(Handle, Handle),
}

impl<Handle> InsertionPoint<Handle> {
    pub fn parent<'a>(&'a self) -> &'a Handle {
        match self {
            InsertionPoint::LastChild(handle) => handle,
            InsertionPoint::BeforeChild(_, handle) => handle,
        }
    }
}

struct ElementPointers<Handle> {
    head: Option<Handle>,
    form: Option<Handle>,
}

impl<Handle> Default for ElementPointers<Handle> {
    fn default() -> ElementPointers<Handle> {
        ElementPointers {
            head: None,
            form: None,
        }
    }
}

pub struct TreeBuilder<Sink: TreeSink> {
    sink: Sink,
    mode: InsertionMode,
    document: Sink::Handle,
    element_pointers: ElementPointers<Sink::Handle>,
    open_elements: Vec<Sink::Handle>,
    foster_parenting: bool,
}

impl<Sink: TreeSink> TreeBuilder<Sink> {
    pub fn new(sink: Sink) -> TreeBuilder<Sink> {
        let document = sink.document();

        TreeBuilder {
            sink,
            mode: InsertionMode::Initial,
            document,
            element_pointers: ElementPointers::default(),
            open_elements: Vec::new(),
            foster_parenting: false,
        }
    }

    fn current_node(&self) -> &Sink::Handle {
        self.open_elements.last().expect("no current node")
    }

    // NOTE: currently this assumes its not a html fragment parser.
    fn adjusted_current_node(&self) -> &Sink::Handle {
        self.current_node()
    }

    fn last_open_element(&self, local_name: &str) -> (usize, Option<&Sink::Handle>) {
        let index = self.open_elements.iter()
            .enumerate()
            .rev()
            .find_map(|(index, handle)| (handle.element_name().local_name == local_name).then(|| index));

        let element = self.open_elements.iter()
            .filter(|handle| handle.element_name().local_name == local_name)
            .last();

        (index.unwrap_or_default(), element)
    }

    fn not_foreign(&self, token: &Token) -> bool {
        let element_name = self.adjusted_current_node().element_name();

        self.open_elements.is_empty()
            || element_name.is_namespace("http://www.w3.org/1999/xhtml")
            || (element_name.is_mathml_text_integration_point() && !(token.is_start_tag("mglyph") || token.is_start_tag("malignmark")))
            || (element_name.is_mathml_text_integration_point() && matches!(token, Token::Character(_)))
            || (element_name.is_mathml_annotation_xml() && token.is_start_tag("svg"))
            || (element_name.is_html_integration_point() && matches!(token, Token::Tag(Tag { kind: TagKind::Start, .. }) | Token::Character(_)))
    }

    fn adjusted_insertion_location(&self, target: &Sink::Handle) -> InsertionPoint<Sink::Handle> {
        let name = target.element_name();

        if self.foster_parenting && ["table", "tbody", "tfoot", "thead", "tr"].contains(&name.local_name) {
            let (template_index, template) = self.last_open_element("template");
            let (table_index, table) = self.last_open_element("table");

            if let Some(handle) = template && table_index < template_index {
                InsertionPoint::LastChild(handle.clone())
            } else if table.is_none() {
                InsertionPoint::LastChild(self.open_elements[0].clone())
            } else if let Some(handle) = table && let Some(parent) = handle.parent() {
                InsertionPoint::BeforeChild(handle.clone(), parent.clone())
            } else {
                InsertionPoint::LastChild(self.open_elements[table_index - 1].clone())
            }
        } else {
            InsertionPoint::LastChild(target.clone())
        }
    }

    fn appropriate_insertion_point(&self, override_: Option<&Sink::Handle>) -> InsertionPoint<Sink::Handle> {
        let target = override_.unwrap_or_else(|| self.current_node());

        self.adjusted_insertion_location(&target)
    }

    // TODO: implement will_execute_script for javascript stuff
    fn create_element_for(&mut self, tag: &Tag, namespace: &str, intended_parent: &Sink::Handle) -> Sink::Handle {
        let name = QualifiedName::new_with_ns(&tag.name, namespace);

        let is = tag.attributes.iter()
            .find(|attribute| attribute.name.as_str() == "is")
            .map(|attribute| attribute.value.as_str());

        let registry = intended_parent.custom_element_registry();

        let will_execute_script = self.sink.custom_element_definition(&registry, name, is).is_some();

        let mut element = self.sink.create_element(intended_parent.node_document(), name, is, will_execute_script, &registry);

        for attribute in tag.attributes.iter() {
            let name = QualifiedName::new_with_ns(attribute.name.as_str(), "");

            element.append_attribute(name, attribute.value.as_str());
        }

        if let Some(form) = &self.element_pointers.form {
            if element.element_name().is_form_associated()
                && !self.open_elements.iter().any(|handle| handle.element_name().local_name == "template")
                && (!element.element_name().is_listed() || element.has_attribute(QualifiedName::new_with_ns("form", "")))
                && intended_parent.root() == form.root()
            {
                element.set_associated_form(form.clone());

                element.set_parser_inserted();
            }
        }

        element
    }

    // TODO: custom elements reaction stack
    fn insert_at(&mut self, element: &Sink::Handle, adjusted_insertion_location: InsertionPoint<Sink::Handle>) {
        match adjusted_insertion_location {
            InsertionPoint::LastChild(mut handle) => handle.append(element),
            InsertionPoint::BeforeChild(mut handle, before) => handle.append_before(&before, element),
        }
    }

    fn insert_foreign_element(&mut self, tag: &Tag, namespace: &str, only_add_to_element_stack: bool) -> Sink::Handle {
        let adjusted_insertion_location = self.appropriate_insertion_point(None);

        let element = self.create_element_for(tag, namespace, adjusted_insertion_location.parent());

        if !only_add_to_element_stack {
            self.insert_at(&element, adjusted_insertion_location);
        }

        self.open_elements.push(element.clone());

        element
    }

    fn append_comment(&mut self, content: &str) {
        let comment = self.sink.create_comment(content);

        self.document.append(&comment);
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
                    let element = self.create_element_for(tag, "http://www.w3.org/1999/xhtml", &self.document.clone());

                    self.document.append(&element);

                    self.open_elements.push(element);

                    self.mode = InsertionMode::BeforeHead;
                },
                Token::Tag(tag) if tag.kind == TagKind::End && !["head", "body", "html", "br"].contains(&tag.name.as_str()) => {
                    self.sink.parse_error(format!("unexpected: {:?}", tag));
                },
                _ => {
                    let element = self.sink.create_element(&self.document, QualifiedName::new_with_ns("html", "http://www.w3.org/1999/xhtml"), None, false, &None);

                    self.document.append(&element);

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
                    let element = self.insert_foreign_element(tag, "http://www.w3.org/1999/xhtml", false);

                    self.element_pointers.head.replace(element);

                    self.mode = InsertionMode::InHead;
                },
                Token::Tag(tag) if tag.kind == TagKind::End && !["head", "body", "html", "br"].contains(&tag.name.as_str()) => {
                    self.sink.parse_error(format!("unexpected: {:?}", tag));
                },
                _ => {
                    let tag = Tag::new(TagKind::Start, String::from("head"), false, Vec::new());
                    let element = self.insert_foreign_element(&tag, "http://www.w3.org/1999/xhtml", false);

                    self.element_pointers.head.replace(element);

                    self.reprocess(token, InsertionMode::InHead);
                },
            },
            _ => todo!(),
        }
    }
}

impl<Sink: TreeSink> TokenSink for TreeBuilder<Sink> {
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


