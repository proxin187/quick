mod state;
pub mod token;

use state::{EscapeKind, RawKind, State};
use token::{Tag, TagKind, Token, TokenSink};

use std::cell::RefCell;
use std::rc::Rc;
use std::io::{BufRead, Bytes};

struct Data {
    tag: Rc<RefCell<Tag>>,
    last: Option<Rc<RefCell<Tag>>>,
    temp: String,
    comment: String,
}

impl Data {
    pub fn new() -> Data {
        Data {
            tag: Rc::new(RefCell::new(Tag::new(TagKind::Start))),
            last: None,
            temp: String::new(),
            comment: String::new(),
        }
    }
}

// TODO: we have to implement a macro that allows us to read the buffer

pub struct Tokenizer<Sink: TokenSink, Buf: BufRead> {
    buffer: Bytes<Buf>,
    sink: Sink,
    state: State,
    data: Data,
}

impl<Sink: TokenSink, Buf: BufRead> Tokenizer<Sink, Buf> {
    pub fn new(buffer: Buf, sink: Sink) -> Tokenizer<Sink, Buf> {
        Tokenizer {
            buffer: buffer.bytes(),
            sink,
            state: State::Data,
            data: Data::new(),
        }
    }

    #[inline]
    fn reconsume(&mut self, input: char, state: State) {
        self.state = state;

        self.step(input);
    }

    #[inline]
    fn bogus_comment(&mut self, input: char) {
        self.sink.emit([Token::Comment("")]);

        self.reconsume(input, State::BogusComment);
    }

    #[inline]
    fn set_state_and_emit<'a, T: IntoIterator<Item = Token<'a>>>(
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

    // TODO: handle eof
    pub fn end(&self) {}

    pub fn step(&mut self, input: char) {
        match self.state {
            // https://html.spec.whatwg.org/multipage/parsing.html#data-state
            State::Data => match input {
                '&' => todo!(),
                '<' => self.state = State::TagOpen,
                c => self.sink.emit([Token::CharacterToken(c)]),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-state
            State::RawData(kind) => match input {
                '&' if kind == RawKind::RcData => todo!(),
                '<' => self.state = State::RawLessThanSign(kind),
                '\0' => self.sink.emit([Token::CharacterToken('\u{fffd}')]),
                c => self.sink.emit([Token::CharacterToken(c)]),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#plaintext-state
            State::Plaintext => match input {
                '\0' => self.sink.emit([Token::CharacterToken('\u{fffd}')]),
                c => self.sink.emit([Token::CharacterToken(c)]),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#tag-open-state
            State::TagOpen => match input {
                '!' => self.state = State::MarkupDeclarationOpen,
                '/' => self.state = State::EndTagOpen,
                '?' => self.bogus_comment(input),
                c if c.is_ascii_alphabetic() => {
                    self.data.tag = Rc::new(RefCell::new(Tag::new(TagKind::Start)));

                    self.reconsume(input, State::TagName);
                },
                _ => {
                    self.sink.emit([Token::CharacterToken('<')]);

                    self.reconsume(input, State::Data);
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#end-tag-open-state
            State::EndTagOpen => match input {
                c if c.is_ascii_alphabetic() => {
                    self.data.tag = Rc::new(RefCell::new(Tag::new(TagKind::End)));

                    self.reconsume(input, State::TagName);
                },
                '>' => self.state = State::Data,
                _ => self.bogus_comment(input),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#tag-name-state
            State::TagName => match input {
                '\u{0009}' | '\u{000a}' | '\u{000c}' | '\u{0020}' => self.state = State::BeforeAttributeName,
                '/' => self.state = State::SelfClosingStartTag,
                '>' => {
                    self.state = State::Data;

                    self.emit_tag();
                },
                '\0' => self.data.tag.borrow_mut().append_name('\u{fffd}'),
                c => self.data.tag.borrow_mut().append_name(c.to_ascii_lowercase()),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-less-than-sign-state
            State::RawLessThanSign(kind) => match input {
                '/' => {
                    self.data.temp.drain(..);

                    self.state = State::RawEndTagOpen(kind);
                },
                '!' if kind == RawKind::ScriptData => self.set_state_and_emit(
                    State::ScriptDataEscapeStart(EscapeKind::Escaped),
                    [Token::CharacterToken('<'), Token::CharacterToken('!')],
                ),
                _ => {
                    self.sink.emit([Token::CharacterToken('<')]);

                    self.reconsume(input, State::RawData(kind));
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-end-tag-open-state
            State::RawEndTagOpen(kind) => match input {
                c if c.is_ascii_alphabetic() => {
                    self.data.tag = Rc::new(RefCell::new(Tag::new(TagKind::End)));

                    self.reconsume(input, State::RawEndTagName(kind));
                },
                _ => {
                    self.sink
                        .emit([Token::CharacterToken('<'), Token::CharacterToken('/')]);

                    self.reconsume(input, State::RawData(kind));
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-end-tag-name-state
            State::RawEndTagName(kind) => match input {
                '\u{0009}' | '\u{000a}' | '\u{000c}' | '\u{0020}'
                    if self.data.tag.borrow().has_appropriate_end_tag(self.data.last.clone()) => self.state = State::BeforeAttributeName,
                '/' if self.data.tag.borrow().has_appropriate_end_tag(self.data.last.clone()) => self.state = State::SelfClosingStartTag,
                '>' if self.data.tag.borrow().has_appropriate_end_tag(self.data.last.clone()) => {
                    self.state = State::Data;

                    self.emit_tag();
                },
                c if c.is_ascii_alphabetic() => {
                    self.data.tag.borrow_mut().append_name(c.to_ascii_lowercase());

                    self.data.temp.push(c);
                }
                _ => {
                    self.sink
                        .emit([Token::CharacterToken('<'), Token::CharacterToken('/')]);

                    self.sink
                        .emit(self.data.temp.chars().map(|c| Token::CharacterToken(c)));

                    self.reconsume(input, State::RawData(kind));
                },
            },

            State::ScriptDataEscapeStart(kind) => match kind {
                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escape-start-state
                EscapeKind::Escaped => match input {
                    '-' => self.set_state_and_emit(State::ScriptDataEscapeStartDash, [Token::CharacterToken('-')]),
                    _ => self.reconsume(input, State::RawData(RawKind::ScriptData)),
                },

                // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escape-start-state
                EscapeKind::DoubleEscaped => match input {
                    '\t' | '\n' | '\x0C' | ' ' | '/' | '>' => if self.data.temp.as_str() == "script" {
                        self.set_state_and_emit(State::ScriptDataEscaped(EscapeKind::DoubleEscaped), [Token::CharacterToken(input)]);
                    } else {
                        self.set_state_and_emit(State::ScriptDataEscaped(EscapeKind::Escaped), [Token::CharacterToken(input)]);
                    },
                    c if c.is_ascii_alphabetic() => {
                        self.data.temp.push(c.to_ascii_lowercase());

                        self.sink.emit([Token::CharacterToken(c)]);
                    },
                    _ => self.reconsume(input, State::ScriptDataEscaped(EscapeKind::Escaped)),
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escape-start-dash-state
            State::ScriptDataEscapeStartDash => match input {
                '-' => self.set_state_and_emit(State::ScriptDataEscapedDashDash(EscapeKind::Escaped), [Token::CharacterToken('-')]),
                _ => self.reconsume(input, State::RawData(RawKind::ScriptData)),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-state
            State::ScriptDataEscaped(kind) => match input {
                '-' => self.set_state_and_emit(State::ScriptDataEscapedDash(kind), [Token::CharacterToken('-')]),
                '<' => {
                    self.state = State::ScriptDataEscapedLessThanSign(kind);

                    if kind == EscapeKind::DoubleEscaped {
                        self.sink.emit([Token::CharacterToken('<')]);
                    }
                }
                '\0' => self.sink.emit([Token::CharacterToken('\u{fffd}')]),
                _ => self.sink.emit([Token::CharacterToken(input)]),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-dash-state
            State::ScriptDataEscapedDash(kind) => match input {
                '-' => self.set_state_and_emit(State::ScriptDataEscapedDashDash(kind), [Token::CharacterToken('-')]),
                '<' => {
                    self.state = State::ScriptDataEscapedLessThanSign(kind);

                    if kind == EscapeKind::DoubleEscaped {
                        self.sink.emit([Token::CharacterToken('<')]);
                    }
                },
                '\0' => self.set_state_and_emit(State::ScriptDataEscaped(kind), [Token::CharacterToken('\u{fffd}')]),
                _ => self.set_state_and_emit(State::ScriptDataEscaped(kind), [Token::CharacterToken(input)]),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-dash-dash-state
            State::ScriptDataEscapedDashDash(kind) => match input {
                '-' => self.sink.emit([Token::CharacterToken('-')]),
                '<' => {
                    self.state = State::ScriptDataEscapedLessThanSign(kind);

                    if kind == EscapeKind::DoubleEscaped {
                        self.sink.emit([Token::CharacterToken('<')]);
                    }
                },
                '>' => self.set_state_and_emit(State::RawData(RawKind::ScriptData), [Token::CharacterToken('>')]),
                '\0' => self.set_state_and_emit(State::ScriptDataEscaped(kind), [Token::CharacterToken('\u{fffd}')]),
                _ => self.set_state_and_emit(State::ScriptDataEscaped(kind), [Token::CharacterToken(input)]),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-less-than-sign-state
            State::ScriptDataEscapedLessThanSign(kind) => match input {
                '/' => {
                    self.data.temp.drain(..);

                    self.state = State::ScriptDataEscapedEndTagOpen;
                },
                c if c.is_ascii_alphabetic() && kind == EscapeKind::Escaped => {
                    self.data.temp.drain(..);

                    self.sink.emit([Token::CharacterToken('<')]);

                    self.reconsume(input, State::ScriptDataEscapeStart(EscapeKind::DoubleEscaped));
                },
                _ => {
                    if kind == EscapeKind::Escaped {
                        self.sink.emit([Token::CharacterToken('<')]);
                    }

                    self.reconsume(input, State::ScriptDataEscaped(kind));
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-end-tag-open-state
            State::ScriptDataEscapedEndTagOpen => match input {
                c if c.is_ascii_alphabetic() => {
                    self.data.tag = Rc::new(RefCell::new(Tag::new(TagKind::End)));

                    self.reconsume(input, State::ScriptDataEscapedEndTagName);
                },
                _ => {
                    self.sink.emit([Token::CharacterToken('<'), Token::CharacterToken('/')]);

                    self.reconsume(input, State::ScriptDataEscaped(EscapeKind::Escaped));
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-end-tag-name-state
            State::ScriptDataEscapedEndTagName => match input {
                '\t' | '\n' | '\x0C' | ' ' if self.data.tag.borrow().has_appropriate_end_tag(self.data.last.clone()) => self.state = State::BeforeAttributeName,
                '/' if self.data.tag.borrow().has_appropriate_end_tag(self.data.last.clone()) => self.state = State::SelfClosingStartTag,
                '>' if self.data.tag.borrow().has_appropriate_end_tag(self.data.last.clone()) => {
                    self.state = State::Data;

                    self.emit_tag();
                },
                c if c.is_ascii_alphabetic() => {
                    self.data.tag.borrow_mut().append_name(c.to_ascii_lowercase());

                    self.data.temp.push(c);
                },
                _ => {
                    self.sink.emit([Token::CharacterToken('<'), Token::CharacterToken('/')]);

                    self.sink.emit(self.data.temp.chars().map(|c| Token::CharacterToken(c)));

                    self.reconsume(input, State::ScriptDataEscaped(EscapeKind::Escaped));
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escape-end-state
            State::ScriptDataDoubleEscapeEnd => match input {
                '\t' | '\n' | '\x0C' | ' ' | '/' | '>' => if self.data.temp.as_str() == "script" {
                    self.set_state_and_emit(State::ScriptDataEscaped(EscapeKind::Escaped), [Token::CharacterToken(input)]);
                } else {
                    self.set_state_and_emit(State::ScriptDataEscaped(EscapeKind::DoubleEscaped), [Token::CharacterToken(input)]);
                },
                c if c.is_ascii_alphabetic() => {
                    self.data.temp.push(c.to_ascii_lowercase());

                    self.sink.emit([Token::CharacterToken(c)]);
                },
                _ => self.reconsume(input, State::ScriptDataEscaped(EscapeKind::DoubleEscaped)),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-name-state
            State::BeforeAttributeName => match input {
                '\t' | '\n' | '\x0C' | ' ' => {},
                '/' | '>' => self.reconsume(input, State::AfterAttributeName),
                '=' => {
                    self.data.tag.borrow_mut().create_attribute(input.to_string(), String::new());

                    self.state = State::AttributeName;
                },
                _ => {
                    self.data.tag.borrow_mut().create_attribute(String::new(), String::new());

                    self.reconsume(input, State::AttributeName);
                }
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state
            State::AttributeName => match input {
                '\t' | '\n' | '\x0C' | ' ' | '/' | '>' => self.reconsume(input, State::AfterAttributeName),
                '=' => self.state = State::BeforeAttributeValue,
                '\0' => self.data.tag.borrow_mut().update_attribute(|attribute| attribute.name.push('\u{fffd}')),
                _ => {
                    let name = input.is_ascii_uppercase()
                        .then(|| input.to_ascii_lowercase())
                        .unwrap_or(input);

                    self.data.tag.borrow_mut().update_attribute(|attribute| attribute.name.push(name));
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-name-state
            State::AfterAttributeName => match input {
                '\t' | '\n' | '\x0C' | ' ' => {},
                '/' => self.state = State::SelfClosingStartTag,
                '=' => self.state = State::BeforeAttributeValue,
                '>' => {
                    self.state = State::Data;

                    self.emit_tag();
                },
                _ => {
                    self.data.tag.borrow_mut().create_attribute(String::new(), String::new());

                    self.reconsume(input, State::AttributeName);
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-value-state
            State::BeforeAttributeValue => match input {
                '\t' | '\n' | '\x0C' | ' ' => {},
                '"' => self.state = State::AttributeValueDoubleQuoted,
                '\'' => self.state = State::AttributeValueSingleQuoted,
                '>' => {
                    self.state = State::Data;

                    self.emit_tag();
                },
                _ => self.reconsume(input, State::AttributeValueUnquoted),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(double-quoted)-state
            State::AttributeValueDoubleQuoted | State::AttributeValueSingleQuoted => match input {
                '"' if self.state == State::AttributeValueDoubleQuoted => self.state = State::AfterAttributeValueQuoted,
                '\'' if self.state == State::AttributeValueSingleQuoted => self.state = State::AfterAttributeValueQuoted,
                '&' => todo!(),
                '\0' => self.data.tag.borrow_mut().update_attribute(|attribute| attribute.value.push('\u{fffd}')),
                _ => self.data.tag.borrow_mut().update_attribute(|attribute| attribute.value.push(input)),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(unquoted)-state
            State::AttributeValueUnquoted => match input {
                '\t' | '\n' | '\x0C' | ' ' => self.state = State::BeforeAttributeName,
                '&' => todo!(),
                '>' => {
                    self.state = State::Data;

                    self.emit_tag();
                },
                '\0' => self.data.tag.borrow_mut().update_attribute(|attribute| attribute.value.push('\u{fffd}')),
                _ => self.data.tag.borrow_mut().update_attribute(|attribute| attribute.value.push(input)),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-value-(quoted)-state
            State::AfterAttributeValueQuoted => match input {
                '\t' | '\n' | '\x0C' | ' ' => self.state = State::BeforeAttributeName,
                '/' => self.state = State::SelfClosingStartTag,
                '>' => {
                    self.state = State::Data;

                    self.emit_tag();
                },
                _ => self.reconsume(input, State::BeforeAttributeName),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#self-closing-start-tag-state
            State::SelfClosingStartTag => match input {
                '>' => {
                    self.data.tag.borrow_mut().self_closing = true;

                    self.state = State::Data;

                    self.emit_tag();
                },
                _ => self.reconsume(input, State::BeforeAttributeName),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#bogus-comment-state
            State::BogusComment => match input {
                '>' => {
                    self.state = State::Data;

                    self.sink.emit([Token::Comment(&self.data.comment)]);
                },
                '\0' => self.data.comment.push('\u{fffd}'),
                _ => self.data.comment.push(input),
            },

            State::MarkupDeclarationOpen => match input {
            },
            _ => unimplemented!(),
        }
    }
}


