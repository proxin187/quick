mod state;
pub mod token;

pub use token::{Doctype, Tag, TagKind, Token, TokenSink};

use state::{DoctypeKind, EscapeKind, RawKind, State};

use std::cell::RefCell;
use std::rc::Rc;
use std::str::Chars;
use std::iter::Peekable;

struct Data {
    doctype: Doctype,
    tag: Rc<RefCell<Tag>>,
    last: Option<Rc<RefCell<Tag>>>,
    temp: String,
    comment: String,
}

impl Data {
    pub fn new() -> Data {
        Data {
            doctype: Doctype::new(),
            tag: Rc::new(RefCell::new(Tag::new(TagKind::Start))),
            last: None,
            temp: String::new(),
            comment: String::new(),
        }
    }
}

struct Buffer<'a> {
    chars: Peekable<Chars<'a>>,
    last: Option<Peekable<Chars<'a>>>,
}

impl<'a> Iterator for Buffer<'a> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        self.last.replace(self.chars.clone());

        self.chars.next()
    }
}

impl<'a> Buffer<'a> {
    pub fn new(chars: Chars<'a>) -> Buffer<'a> {
        Buffer {
            chars: chars.peekable(),
            last: None,
        }
    }

    fn reconsume(&mut self) {
        if let Some(chars) = self.last.take() {
            self.chars = chars;
        }
    }

    fn is_empty(&mut self) -> bool {
        self.chars.peek().is_none()
    }

    // NOTE: this is case insensitive, technically its not completely compliant with the HTML standard.
    fn peek_exact(&mut self, string: &str) -> bool {
        self.chars.clone()
            .take(string.len())
            .map(|c| c.to_ascii_lowercase())
            .eq(string.chars())
            .then(|| self.chars.nth(string.len() - 1))
            .is_some()
    }
}

pub struct Tokenizer<'a, Sink: TokenSink> {
    sink: &'a mut Sink,
    buffer: Buffer<'a>,
    state: State,
    data: Data,
}

impl<'a, Sink: TokenSink> Tokenizer<'a, Sink> {
    pub fn new(sink: &'a mut Sink, chars: Chars<'a>) -> Tokenizer<'a, Sink> {
        Tokenizer {
            sink,
            buffer: Buffer::new(chars),
            state: State::Data,
            data: Data::new(),
        }
    }

    #[inline]
    fn reconsume(&mut self, state: State) {
        self.buffer.reconsume();

        self.state = state;
    }

    #[inline]
    fn bogus_comment(&mut self) {
        self.sink.emit([Token::Comment("")]);

        self.reconsume(State::BogusComment);
    }

    #[inline]
    fn set_state_and_emit<'t, T: IntoIterator<Item = Token<'t>>>(
        &mut self,
        state: State,
        tokens: T,
    ) {
        self.state = state;

        self.sink.emit(tokens);
    }

    // TODO: implement emit tag
    fn emit_tag(&mut self) {
        self.data.last.replace(Rc::clone(&self.data.tag));
    }

    // TODO: implement emit doctype
    fn emit_doctype(&mut self) {
    }

    pub fn wait(&mut self) {
        while !self.buffer.is_empty() {
            self.step();
        }
    }

    pub fn step(&mut self) {
        match self.state {
            // https://html.spec.whatwg.org/multipage/parsing.html#data-state
            State::Data => match self.buffer.next() {
                Some('&') => todo!(),
                Some('<') => self.state = State::TagOpen,
                Some(c) => self.sink.emit([Token::CharacterToken(c)]),
                None => self.sink.eof(),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-state
            State::RawData(kind) => match self.buffer.next() {
                Some('&') if kind == RawKind::RcData => todo!(),
                Some('<') => self.state = State::RawLessThanSign(kind),
                Some('\0') => self.sink.emit([Token::CharacterToken('\u{fffd}')]),
                Some(c) => self.sink.emit([Token::CharacterToken(c)]),
                None => self.sink.eof(),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#plaintext-state
            State::Plaintext => match self.buffer.next() {
                Some('\0') => self.sink.emit([Token::CharacterToken('\u{fffd}')]),
                Some(c) => self.sink.emit([Token::CharacterToken(c)]),
                None => self.sink.eof(),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#tag-open-state
            State::TagOpen => match self.buffer.next() {
                Some('!') => self.state = State::MarkupDeclarationOpen,
                Some('/') => self.state = State::EndTagOpen,
                Some('?') => self.bogus_comment(),
                Some(c) if c.is_ascii_alphabetic() => {
                    self.data.tag = Rc::new(RefCell::new(Tag::new(TagKind::Start)));

                    self.reconsume(State::TagName);
                },
                next => {
                    self.sink.emit([Token::CharacterToken('<')]);

                    if next.is_some() {
                        self.reconsume(State::Data);
                    } else {
                        self.sink.eof();
                    }
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#end-tag-open-state
            State::EndTagOpen => match self.buffer.next() {
                Some(c) if c.is_ascii_alphabetic() => {
                    self.data.tag = Rc::new(RefCell::new(Tag::new(TagKind::End)));

                    self.reconsume(State::TagName);
                },
                Some('>') => self.state = State::Data,
                Some(_) => self.bogus_comment(),
                None => {
                    self.sink.emit([Token::CharacterToken('<'), Token::CharacterToken('/')]);

                    self.sink.eof();
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#tag-name-state
            State::TagName => match self.buffer.next() {
                Some('\u{0009}' | '\u{000a}' | '\u{000c}' | '\u{0020}') => self.state = State::BeforeAttributeName,
                Some('/') => self.state = State::SelfClosingStartTag,
                Some('>') => {
                    self.state = State::Data;

                    self.emit_tag();
                },
                Some('\0') => self.data.tag.borrow_mut().append_name('\u{fffd}'),
                Some(c) => self.data.tag.borrow_mut().append_name(c.to_ascii_lowercase()),
                None => self.sink.eof(),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-less-than-sign-state
            State::RawLessThanSign(kind) => match self.buffer.next() {
                Some('/') => {
                    self.data.temp.drain(..);

                    self.state = State::RawEndTagOpen(kind);
                },
                Some('!') if kind == RawKind::ScriptData => self.set_state_and_emit(
                    State::ScriptDataEscapeStart(EscapeKind::Escaped),
                    [Token::CharacterToken('<'), Token::CharacterToken('!')],
                ),
                _ => {
                    self.sink.emit([Token::CharacterToken('<')]);

                    self.reconsume(State::RawData(kind));
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-end-tag-open-state
            State::RawEndTagOpen(kind) => match self.buffer.next() {
                Some(c) if c.is_ascii_alphabetic() => {
                    self.data.tag = Rc::new(RefCell::new(Tag::new(TagKind::End)));

                    self.reconsume(State::RawEndTagName(kind));
                },
                _ => {
                    self.sink.emit([Token::CharacterToken('<'), Token::CharacterToken('/')]);

                    self.reconsume(State::RawData(kind));
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-end-tag-name-state
            State::RawEndTagName(kind) => match self.buffer.next() {
                Some('\u{0009}' | '\u{000a}' | '\u{000c}' | '\u{0020}')
                    if self.data.tag.borrow().has_appropriate_end_tag(self.data.last.clone()) => self.state = State::BeforeAttributeName,
                Some('/') if self.data.tag.borrow().has_appropriate_end_tag(self.data.last.clone()) => self.state = State::SelfClosingStartTag,
                Some('>') if self.data.tag.borrow().has_appropriate_end_tag(self.data.last.clone()) => {
                    self.state = State::Data;

                    self.emit_tag();
                },
                Some(c) if c.is_ascii_alphabetic() => {
                    self.data.tag.borrow_mut().append_name(c.to_ascii_lowercase());

                    self.data.temp.push(c);
                }
                _ => {
                    self.sink.emit([Token::CharacterToken('<'), Token::CharacterToken('/')]);

                    self.sink.emit(self.data.temp.chars().map(|c| Token::CharacterToken(c)));

                    self.reconsume(State::RawData(kind));
                },
            },

            State::ScriptDataEscapeStart(kind) => match kind {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escape-start-state
                EscapeKind::Escaped => match self.buffer.next() {
                    Some('-') => self.set_state_and_emit(State::ScriptDataEscapeStartDash, [Token::CharacterToken('-')]),
                    _ => self.reconsume(State::RawData(RawKind::ScriptData)),
                },

                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escape-start-state
                EscapeKind::DoubleEscaped => match self.buffer.next() {
                    Some(c) if matches!(c, '\t' | '\n' | '\x0C' | ' ' | '/' | '>') && self.data.temp.as_str() == "script" => {
                        self.set_state_and_emit(State::ScriptDataEscaped(EscapeKind::DoubleEscaped), [Token::CharacterToken(c)]);
                    },
                    Some(c) if matches!(c, '\t' | '\n' | '\x0C' | ' ' | '/' | '>') && self.data.temp.as_str() != "script" => {
                        self.set_state_and_emit(State::ScriptDataEscaped(EscapeKind::Escaped), [Token::CharacterToken(c)]);
                    },
                    Some(c) if c.is_ascii_alphabetic() => {
                        self.data.temp.push(c.to_ascii_lowercase());

                        self.sink.emit([Token::CharacterToken(c)]);
                    },
                    _ => self.reconsume(State::ScriptDataEscaped(EscapeKind::Escaped)),
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escape-start-dash-state
            State::ScriptDataEscapeStartDash => match self.buffer.next() {
                Some('-') => self.set_state_and_emit(State::ScriptDataEscapedDashDash(EscapeKind::Escaped), [Token::CharacterToken('-')]),
                _ => self.reconsume(State::RawData(RawKind::ScriptData)),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-state
            State::ScriptDataEscaped(kind) => match self.buffer.next() {
                Some('-') => self.set_state_and_emit(State::ScriptDataEscapedDash(kind), [Token::CharacterToken('-')]),
                Some('<') => {
                    self.state = State::ScriptDataEscapedLessThanSign(kind);

                    if kind == EscapeKind::DoubleEscaped {
                        self.sink.emit([Token::CharacterToken('<')]);
                    }
                }
                Some('\0') => self.sink.emit([Token::CharacterToken('\u{fffd}')]),
                Some(c) => self.sink.emit([Token::CharacterToken(c)]),
                None => self.sink.eof(),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-dash-state
            State::ScriptDataEscapedDash(kind) => match self.buffer.next() {
                Some('-') => self.set_state_and_emit(State::ScriptDataEscapedDashDash(kind), [Token::CharacterToken('-')]),
                Some('<') => {
                    self.state = State::ScriptDataEscapedLessThanSign(kind);

                    if kind == EscapeKind::DoubleEscaped {
                        self.sink.emit([Token::CharacterToken('<')]);
                    }
                },
                Some('\0') => self.set_state_and_emit(State::ScriptDataEscaped(kind), [Token::CharacterToken('\u{fffd}')]),
                Some(c) => self.set_state_and_emit(State::ScriptDataEscaped(kind), [Token::CharacterToken(c)]),
                None => self.sink.eof(),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-dash-dash-state
            State::ScriptDataEscapedDashDash(kind) => match self.buffer.next() {
                Some('-') => self.sink.emit([Token::CharacterToken('-')]),
                Some('<') => {
                    self.state = State::ScriptDataEscapedLessThanSign(kind);

                    if kind == EscapeKind::DoubleEscaped {
                        self.sink.emit([Token::CharacterToken('<')]);
                    }
                },
                Some('>') => self.set_state_and_emit(State::RawData(RawKind::ScriptData), [Token::CharacterToken('>')]),
                Some('\0') => self.set_state_and_emit(State::ScriptDataEscaped(kind), [Token::CharacterToken('\u{fffd}')]),
                Some(c) => self.set_state_and_emit(State::ScriptDataEscaped(kind), [Token::CharacterToken(c)]),
                None => self.sink.eof(),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-less-than-sign-state
            State::ScriptDataEscapedLessThanSign(kind) => match self.buffer.next() {
                Some('/') => {
                    self.data.temp.drain(..);

                    self.state = State::ScriptDataEscapedEndTagOpen;
                },
                Some(c) if c.is_ascii_alphabetic() && kind == EscapeKind::Escaped => {
                    self.data.temp.drain(..);

                    self.sink.emit([Token::CharacterToken('<')]);

                    self.reconsume(State::ScriptDataEscapeStart(EscapeKind::DoubleEscaped));
                },
                _ => {
                    if kind == EscapeKind::Escaped {
                        self.sink.emit([Token::CharacterToken('<')]);
                    }

                    self.reconsume(State::ScriptDataEscaped(kind));
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-end-tag-open-state
            State::ScriptDataEscapedEndTagOpen => match self.buffer.next() {
                Some(c) if c.is_ascii_alphabetic() => {
                    self.data.tag = Rc::new(RefCell::new(Tag::new(TagKind::End)));

                    self.reconsume(State::ScriptDataEscapedEndTagName);
                },
                _ => {
                    self.sink.emit([Token::CharacterToken('<'), Token::CharacterToken('/')]);

                    self.reconsume(State::ScriptDataEscaped(EscapeKind::Escaped));
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-end-tag-name-state
            State::ScriptDataEscapedEndTagName => match self.buffer.next() {
                Some('\t' | '\n' | '\x0C' | ' ') if self.data.tag.borrow().has_appropriate_end_tag(self.data.last.clone()) => self.state = State::BeforeAttributeName,
                Some('/') if self.data.tag.borrow().has_appropriate_end_tag(self.data.last.clone()) => self.state = State::SelfClosingStartTag,
                Some('>') if self.data.tag.borrow().has_appropriate_end_tag(self.data.last.clone()) => {
                    self.state = State::Data;

                    self.emit_tag();
                },
                Some(c) if c.is_ascii_alphabetic() => {
                    self.data.tag.borrow_mut().append_name(c.to_ascii_lowercase());

                    self.data.temp.push(c);
                },
                _ => {
                    self.sink.emit([Token::CharacterToken('<'), Token::CharacterToken('/')]);

                    self.sink.emit(self.data.temp.chars().map(|c| Token::CharacterToken(c)));

                    self.reconsume(State::ScriptDataEscaped(EscapeKind::Escaped));
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escape-end-state
            State::ScriptDataDoubleEscapeEnd => match self.buffer.next() {
                Some(c) if matches!(c, '\t' | '\n' | '\x0C' | ' ' | '/' | '>') && self.data.temp.as_str() == "script" => {
                    self.set_state_and_emit(State::ScriptDataEscaped(EscapeKind::Escaped), [Token::CharacterToken(c)]);
                },
                Some(c) if matches!(c, '\t' | '\n' | '\x0C' | ' ' | '/' | '>') && self.data.temp.as_str() != "script" => {
                    self.set_state_and_emit(State::ScriptDataEscaped(EscapeKind::DoubleEscaped), [Token::CharacterToken(c)]);
                },
                Some(c) if c.is_ascii_alphabetic() => {
                    self.data.temp.push(c.to_ascii_lowercase());

                    self.sink.emit([Token::CharacterToken(c)]);
                },
                _ => self.reconsume(State::ScriptDataEscaped(EscapeKind::DoubleEscaped)),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-name-state
            State::BeforeAttributeName => match self.buffer.next() {
                Some('\t' | '\n' | '\x0C' | ' ') => {},
                Some('/' | '>') => self.reconsume(State::AfterAttributeName),
                Some('=') => {
                    self.data.tag.borrow_mut().create_attribute(String::from('='), String::new());

                    self.state = State::AttributeName;
                },
                Some(_) => {
                    self.data.tag.borrow_mut().create_attribute(String::new(), String::new());

                    self.reconsume(State::AttributeName);
                },
                None => self.reconsume(State::AfterAttributeName),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state
            State::AttributeName => match self.buffer.next() {
                Some('\t' | '\n' | '\x0C' | ' ' | '/' | '>') => self.reconsume(State::AfterAttributeName),
                Some('=') => self.state = State::BeforeAttributeValue,
                Some('\0') => self.data.tag.borrow_mut().update_attribute(|attribute| attribute.name.push('\u{fffd}')),
                Some(c) => {
                    let name = c.is_ascii_uppercase()
                        .then(|| c.to_ascii_lowercase())
                        .unwrap_or(c);

                    self.data.tag.borrow_mut().update_attribute(|attribute| attribute.name.push(name));
                },
                None => self.reconsume(State::AfterAttributeName),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-name-state
            State::AfterAttributeName => match self.buffer.next() {
                Some('\t' | '\n' | '\x0C' | ' ') => {},
                Some('/') => self.state = State::SelfClosingStartTag,
                Some('=') => self.state = State::BeforeAttributeValue,
                Some('>') => {
                    self.state = State::Data;

                    self.emit_tag();
                },
                Some(_) => {
                    self.data.tag.borrow_mut().create_attribute(String::new(), String::new());

                    self.reconsume(State::AttributeName);
                },
                None => self.sink.eof(),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-value-state
            State::BeforeAttributeValue => match self.buffer.next() {
                Some('\t' | '\n' | '\x0C' | ' ') => {},
                Some('"') => self.state = State::AttributeValueDoubleQuoted,
                Some('\'') => self.state = State::AttributeValueSingleQuoted,
                Some('>') => {
                    self.state = State::Data;

                    self.emit_tag();
                },
                _ => self.reconsume(State::AttributeValueUnquoted),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(double-quoted)-state
            State::AttributeValueDoubleQuoted | State::AttributeValueSingleQuoted => match self.buffer.next() {
                Some('"') if self.state == State::AttributeValueDoubleQuoted => self.state = State::AfterAttributeValueQuoted,
                Some('\'') if self.state == State::AttributeValueSingleQuoted => self.state = State::AfterAttributeValueQuoted,
                Some('&') => todo!(),
                Some('\0') => self.data.tag.borrow_mut().update_attribute(|attribute| attribute.value.push('\u{fffd}')),
                Some(c) => self.data.tag.borrow_mut().update_attribute(|attribute| attribute.value.push(c)),
                None => self.sink.eof(),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(unquoted)-state
            State::AttributeValueUnquoted => match self.buffer.next() {
                Some('\t' | '\n' | '\x0C' | ' ') => self.state = State::BeforeAttributeName,
                Some('&') => todo!(),
                Some('>') => {
                    self.state = State::Data;

                    self.emit_tag();
                },
                Some('\0') => self.data.tag.borrow_mut().update_attribute(|attribute| attribute.value.push('\u{fffd}')),
                Some(c) => self.data.tag.borrow_mut().update_attribute(|attribute| attribute.value.push(c)),
                None => self.sink.eof(),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-value-(quoted)-state
            State::AfterAttributeValueQuoted => match self.buffer.next() {
                Some('\t' | '\n' | '\x0C' | ' ') => self.state = State::BeforeAttributeName,
                Some('/') => self.state = State::SelfClosingStartTag,
                Some('>') => {
                    self.state = State::Data;

                    self.emit_tag();
                },
                Some(_) => self.reconsume(State::BeforeAttributeName),
                None => self.sink.eof(),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#self-closing-start-tag-state
            State::SelfClosingStartTag => match self.buffer.next() {
                Some('>') => {
                    self.data.tag.borrow_mut().self_closing = true;

                    self.state = State::Data;

                    self.emit_tag();
                },
                Some(_) => self.reconsume(State::BeforeAttributeName),
                None => self.sink.eof(),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#bogus-comment-state
            State::BogusComment => match self.buffer.next() {
                Some('>') => {
                    self.state = State::Data;

                    self.sink.emit([Token::Comment(&self.data.comment)]);
                },
                Some('\0') => self.data.comment.push('\u{fffd}'),
                Some(c) => self.data.comment.push(c),
                None => self.sink.eof(),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#markup-declaration-open-state
            //
            // NOTE: cloning the buffer is cheap as it only clones the iterator state and not the data itself.
            State::MarkupDeclarationOpen => {
                if self.buffer.peek_exact("--") {
                    self.data.comment.drain(..);

                    self.state = State::CommentStart;
                } else if self.buffer.peek_exact("doctype") {
                    self.state = State::Doctype;
                } else if self.buffer.peek_exact("[cdata[") {
                    // FIXME: we use peek_exact which is case insensitive, technically its
                    // supposed to be case sensitive here.

                    if self.sink.adjusted_node_namespace() {
                        self.state = State::CDataSection;
                    } else {
                        self.data.comment = String::from("[CDATA[");

                        self.state = State::BogusComment;
                    }
                } else {
                    self.data.comment.drain(..);

                    self.state = State::BogusComment;
                }
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#comment-start-state
            State::CommentStart => match self.buffer.next() {
                Some('-') => self.state = State::CommentStartDash,
                Some('>') => {
                    self.sink.emit([Token::Comment(&self.data.comment)]);

                    self.state = State::Data;
                },
                _ => self.reconsume(State::Comment),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#comment-start-dash-state
            State::CommentStartDash => match self.buffer.next() {
                Some('-') => self.state = State::CommentEnd,
                Some('>') => {
                    self.sink.emit([Token::Comment(&self.data.comment)]);

                    self.state = State::Data;
                },
                Some(_) => {
                    self.data.comment.push('-');

                    self.reconsume(State::Comment);
                },
                None => {
                    self.sink.emit([Token::Comment(&self.data.comment)]);

                    self.sink.eof();
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#comment-state
            State::Comment => match self.buffer.next() {
                Some('<') => {
                    self.data.comment.push('<');

                    self.state = State::CommentLessThenSign;
                },
                Some('-') => self.state = State::CommentEndDash,
                Some('\0') => self.data.comment.push('\u{fffd}'),
                Some(c) => self.data.comment.push(c),
                None => {
                    self.sink.emit([Token::Comment(&self.data.comment)]);

                    self.sink.eof();
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-state
            State::CommentLessThenSign => match self.buffer.next() {
                Some('!') => {
                    self.data.comment.push('!');

                    self.state = State::CommentLessThenSignBang;
                },
                Some('<') => self.data.comment.push('<'),
                _ => self.reconsume(State::Comment),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-bang-state
            State::CommentLessThenSignBang => match self.buffer.next() {
                Some('-') => self.state = State::CommentLessThenSignBangDash,
                _ => self.reconsume(State::Comment),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-bang-dash-state
            State::CommentLessThenSignBangDash => match self.buffer.next() {
                Some('-') => self.state = State::CommentLessThenSignBangDashDash,
                _ => self.reconsume(State::CommentEndDash),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#comment-less-than-sign-bang-dash-dash-state
            State::CommentLessThenSignBangDashDash => self.reconsume(State::CommentEnd),

            // https://html.spec.whatwg.org/multipage/parsing.html#comment-end-dash-state
            State::CommentEndDash => match self.buffer.next() {
                Some('-') => self.state = State::CommentEnd,
                Some(_) => {
                    self.data.comment.push('-');

                    self.reconsume(State::Comment);
                },
                None => {
                    self.sink.emit([Token::Comment(&self.data.comment)]);

                    self.sink.eof();
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#comment-end-state
            State::CommentEnd => match self.buffer.next() {
                Some('>') => {
                    self.sink.emit([Token::Comment(&self.data.comment)]);

                    self.state = State::Data;
                },
                Some('!') => self.state = State::CommentEndBang,
                Some('-') => self.data.comment.push('-'),
                Some(_) => {
                    self.data.comment.push_str("--");

                    self.reconsume(State::Comment);
                },
                None => {
                    self.sink.emit([Token::Comment(&self.data.comment)]);

                    self.sink.eof();
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#comment-end-bang-state
            State::CommentEndBang => match self.buffer.next() {
                Some('-') => {
                    self.data.comment.push_str("--!");

                    self.state = State::CommentEndDash;
                },
                Some('>') => {
                    self.sink.emit([Token::Comment(&self.data.comment)]);

                    self.state = State::Data;
                },
                Some(_) => {
                    self.data.comment.push_str("--!");

                    self.reconsume(State::Comment);
                },
                None => {
                    self.sink.emit([Token::Comment(&self.data.comment)]);

                    self.sink.eof();
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#doctype-state
            State::Doctype => match self.buffer.next() {
                Some('\t' | '\n' | '\x0C' | ' ') => self.state = State::BeforeDoctypeName,
                Some(_) => self.reconsume(State::BeforeDoctypeName),
                None => {
                    self.data.doctype.reset();

                    self.data.doctype.force_quirks = true;

                    self.emit_doctype();

                    self.sink.eof();
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-name-state
            State::BeforeDoctypeName => match self.buffer.next() {
                Some('\t' | '\n' | '\x0C' | ' ') => {},
                Some('\0') => {
                    self.data.doctype.reset();

                    self.data.doctype.name.append('\u{fffd}');

                    self.state = State::DoctypeName;
                },
                Some('>') => {
                    self.data.doctype.reset();

                    self.data.doctype.force_quirks = true;

                    self.emit_doctype();

                    self.state = State::Data;
                },
                Some(c) => {
                    self.data.doctype.reset();

                    self.data.doctype.name.append(c.to_ascii_lowercase());

                    self.state = State::DoctypeName;
                },
                None => {
                    self.data.doctype.reset();

                    self.data.doctype.force_quirks = true;

                    self.emit_doctype();

                    self.sink.eof();
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#doctype-name-state
            State::DoctypeName => match self.buffer.next() {
                Some('\t' | '\n' | '\x0C' | ' ') => self.state = State::AfterDoctypeName,
                Some('>') => {
                    self.emit_doctype();

                    self.state = State::Data;
                },
                Some('\0') => self.data.doctype.name.append('\u{fffd}'),
                Some(c) => self.data.doctype.name.append(c.to_ascii_lowercase()),
                None => {
                    self.data.doctype.force_quirks = true;

                    self.emit_doctype();

                    self.sink.eof();
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-name-state
            State::AfterDoctypeName => match self.buffer.next() {
                Some('\t' | '\n' | '\x0C' | ' ') => {},
                Some('>') => {
                    self.emit_doctype();

                    self.state = State::Data;
                },
                Some('p' | 'P') if self.buffer.peek_exact("ublic") => self.state = State::AfterDoctypeKeyword(DoctypeKind::Public),
                Some('s' | 'S') if self.buffer.peek_exact("ystem") => self.state = State::AfterDoctypeKeyword(DoctypeKind::System),
                Some(_) => {
                    self.data.doctype.force_quirks = true;

                    self.reconsume(State::BogusDoctype);
                },
                None => {
                    self.data.doctype.force_quirks = true;

                    self.emit_doctype();

                    self.sink.eof();
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#after-doctype-public-keyword-state
            State::AfterDoctypeKeyword(kind) => match self.buffer.next() {
                Some('\t' | '\n' | '\x0C' | ' ') => self.state = State::BeforeDoctypeIdentifier(kind),
                Some(c) if matches!(c, '"' | '\'') => {
                    self.data.doctype.get_id(kind).drain();

                    if c == '"' {
                        self.state = State::DoctypeIdentifierDoubleQuoted(kind);
                    } else {
                        self.state = State::DoctypeIdentifierSingleQuoted(kind);
                    }
                },
                Some('>') => {
                    self.data.doctype.force_quirks = true;

                    self.emit_doctype();

                    self.state = State::Data;
                },
                Some(_) => {
                    self.data.doctype.force_quirks = true;

                    self.reconsume(State::BogusDoctype);
                },
                None => {
                    self.data.doctype.force_quirks = true;

                    self.emit_doctype();

                    self.sink.eof();
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#before-doctype-public-identifier-state
            State::BeforeDoctypeIdentifier(kind) => match self.buffer.next() {
                Some('\t' | '\n' | '\x0C' | ' ') => {},
                Some(c) if matches!(c, '"' | '\'') => {
                    self.data.doctype.get_id(kind).drain();

                    if c == '"' {
                        self.state = State::DoctypeIdentifierDoubleQuoted(kind);
                    } else {
                        self.state = State::DoctypeIdentifierSingleQuoted(kind);
                    }
                },
                Some('>') => {
                    self.data.doctype.force_quirks = true;

                    self.emit_doctype();

                    self.state = State::Data;
                },
                Some(_) => {
                    self.data.doctype.force_quirks = true;

                    self.reconsume(State::BogusDoctype)
                },
                None => {
                    self.data.doctype.force_quirks = true;

                    self.emit_doctype();

                    self.sink.eof();
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#doctype-public-identifier-(double-quoted)-state
            State::DoctypeIdentifierDoubleQuoted(kind) => match self.buffer.next() {
                Some('"') => self.state = State::AfterDoctypeIdentifier(kind),
                Some('\0') => self.data.doctype.get_id(kind).append('\u{fffd}'),
                Some('>') => {
                    self.data.doctype.force_quirks = true;

                    self.emit_doctype();

                    self.state = State::Data;
                },
                Some(c) => self.data.doctype.get_id(kind).append(c),
                None => {
                    self.data.doctype.force_quirks = true;

                    self.emit_doctype();

                    self.sink.eof();
                },
            },

            // TODO: remove the repeating code.
            _ => unimplemented!(),
        }
    }
}


