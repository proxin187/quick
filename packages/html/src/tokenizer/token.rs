use std::cell::RefCell;
use std::rc::Rc;


#[derive(Debug)]
pub enum TagKind {
    Start,
    End,
}

#[derive(Debug)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct Tag {
    pub kind: TagKind,
    pub name: String,
    pub self_closing: bool,
    pub attributes: Vec<Attribute>,
}

impl Tag {
    pub fn new(kind: TagKind) -> Tag {
        Tag {
            kind,
            name: String::new(),
            self_closing: false,
            attributes: Vec::new(),
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

#[derive(Debug)]
pub enum Token<'a> {
    Tag(Tag),
    CharacterToken(char),
    Comment(&'a str),
}

pub trait TokenSink {
    fn process(&mut self, token: Token);

    fn eof(&self);

    fn emit<'a, T: IntoIterator<Item = Token<'a>>>(&mut self, tokens: T) {
        for token in tokens {
            self.process(token);
        }
    }

    fn adjusted_node_namespace(&self) -> bool { false }
}


