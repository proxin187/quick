use super::DoctypeKind;

use std::cell::RefCell;
use std::rc::Rc;


#[derive(Debug)]
pub struct DoctypeValue {
    value: Option<String>,
}

impl DoctypeValue {
    pub fn new() -> DoctypeValue {
        DoctypeValue {
            value: None,
        }
    }

    pub(super) fn append(&mut self, character: char) {
        if let Some(value) = &mut self.value {
            value.push(character);
        } else {
            self.value.replace(character.to_string());
        }
    }

    pub(super) fn drain(&mut self) {
        if let Some(value) = &mut self.value {
            value.drain(..);
        }
    }
}

#[derive(Debug)]
pub struct Doctype {
    pub name: DoctypeValue,
    pub public_id: DoctypeValue,
    pub system_id: DoctypeValue,
    pub force_quirks: bool,
}

impl Doctype {
    pub fn new() -> Doctype {
        Doctype {
            name: DoctypeValue::new(),
            public_id: DoctypeValue::new(),
            system_id: DoctypeValue::new(),
            force_quirks: false,
        }
    }

    pub(super) fn get_id(&mut self, kind: DoctypeKind) -> &mut DoctypeValue {
        match kind {
            DoctypeKind::Public => &mut self.public_id,
            DoctypeKind::System => &mut self.system_id,
        }
    }

    pub(super) fn reset(&mut self) {
        *self = Doctype::new();
    }
}

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
    Doctype(Doctype),
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


