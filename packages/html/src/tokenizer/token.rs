use std::cell::RefCell;
use std::rc::Rc;

pub struct Tag {
    kind: TagKind,
    name: String,
}

impl Tag {
    pub fn new(kind: TagKind) -> Tag {
        Tag {
            kind,
            name: String::new(),
        }
    }

    pub(super) fn append_name(&mut self, character: char) {
        self.name.push(character);
    }

    pub(super) fn has_appropriate_end_tag(&self, last: Option<Rc<RefCell<Tag>>>) -> bool {
        last.map(|tag| tag.borrow().name == self.name)
            .unwrap_or_default()
    }
}

pub enum TagKind {
    Start,
    End,
}

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
}


