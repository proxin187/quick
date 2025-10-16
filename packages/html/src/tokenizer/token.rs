use std::cell::RefCell;
use std::rc::Rc;

use unicase::UniCase;


#[derive(Debug, PartialEq)]
pub struct Doctype<'a> {
    pub name: Option<UniCase<&'a str>>,
    pub public_id: Option<UniCase<&'a str>>,
    pub system_id: Option<UniCase<&'a str>>,
    pub force_quirks: bool,
}

impl<'a> Doctype<'a> {
    #[inline]
    pub fn is_parse_error(&self) -> bool {
        self.name != Some(UniCase::new("html"))
            || self.public_id.is_some()
            || (self.system_id.is_some() && self.system_id != Some(UniCase::new("about:legacy-compat")))
    }
}

#[derive(Debug, PartialEq)]
pub enum TagKind {
    Start,
    End,
}

#[derive(Debug, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct Tag {
    pub kind: TagKind,
    pub name: String,
    pub self_closing: bool,
    pub attributes: Vec<Attribute>,
}

impl Tag {
    pub fn new(kind: TagKind, name: String, self_closing: bool, attributes: Vec<Attribute>) -> Tag {
        Tag {
            kind,
            name,
            self_closing,
            attributes,
        }
    }

    pub(super) fn update_attribute(&mut self, f: impl Fn(&mut Attribute)) {
        let len = self.attributes.len();

        f(&mut self.attributes[len - 1])
    }

    pub(super) fn create_attribute(&mut self, name: String, value: String) {
        self.attributes.push(Attribute { name, value });
    }

    pub(super) fn append_name(&mut self, character: char) {
        self.name.push(character);
    }

    pub(super) fn has_appropriate_end_tag(&self, last: Option<Rc<RefCell<Tag>>>) -> bool {
        last.map(|tag| tag.borrow().name == self.name)
            .unwrap_or_default()
    }
}

#[derive(Debug, PartialEq)]
/// Represents a Token.
pub enum Token<'a> {
    Tag(&'a Tag),
    Doctype(Doctype<'a>),
    Character(char),
    Comment(&'a str),
}

impl<'a> Token<'a> {
    pub fn is_start_tag(&self, name: &str) -> bool {
        match self {
            Token::Tag(tag) =>  tag.name.as_str() == name,
            _ => false,
        }
    }
}

/// Recieve tokens from the tokenizer in the TokenSink.
pub trait TokenSink {
    fn process(&mut self, token: Token);

    fn eof(&mut self);

    fn emit<'a, T: IntoIterator<Item = Token<'a>>>(&mut self, tokens: T) {
        for token in tokens {
            self.process(token);
        }
    }

    fn adjusted_node_namespace(&self) -> bool { false }
}


