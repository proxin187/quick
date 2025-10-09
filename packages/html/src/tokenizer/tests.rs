use crate::tokenizer::{Tokenizer, Token, TokenSink, Tag, TagKind, Attribute};

use std::fs;


struct Sink<'a> {
    expected: Vec<Token<'a>>,
}

impl<'a> Sink<'a> {
    pub fn new() -> Sink<'a> {
        Sink {
            expected: Vec::new(),
        }
    }

    fn push_characters(&mut self, string: &str) {
        self.expected.extend(string.chars().map(|c| Token::CharacterToken(c)));
    }

    fn push_tag<const N: usize>(&mut self, kind: TagKind, name: &str, self_closing: bool, attributes: [Attribute; N]) {
        let tag = Box::new(Tag { kind, name: name.to_string(), self_closing, attributes: attributes.into_iter().collect() });

        self.expected.push(Token::Tag(Box::leak(tag)));
    }
}

impl<'a> TokenSink for Sink<'a> {
    fn process(&mut self, token: Token) {
        assert_eq!(self.expected.pop(), Some(token));
    }

    fn eof(&self) {
        println!("end of file");
    }
}

// TODO: it doesnt look like a good idea to manually push all the expected tokens, rather we should
// have it stored in a file and compare
#[test]
fn small_fragment() -> Result<(), Box<dyn std::error::Error>> {
    let html = fs::read_to_string("data/small-fragment.html")?;

    let mut sink = Sink::new();

    sink.push_tag(TagKind::Start, "p", false, []);

    let mut tokenizer = Tokenizer::new(&mut sink, html.chars());

    while !tokenizer.step() {}

    Ok(())
}


