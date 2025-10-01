mod state;
pub mod token;

use state::{EscapeKind, RawKind, State};
use token::{Tag, TagKind, Token, TokenSink};

use std::cell::RefCell;
use std::rc::Rc;

pub struct Tokenizer<Sink: TokenSink> {
    sink: Sink,
    state: State,
    tag: Rc<RefCell<Tag>>,
    last: Option<Rc<RefCell<Tag>>>,
    temp: String,
}

impl<Sink: TokenSink> Tokenizer<Sink> {
    pub fn new(sink: Sink) -> Tokenizer<Sink> {
        Tokenizer {
            sink,
            state: State::Data,
            tag: Rc::new(RefCell::new(Tag::new(TagKind::Start))),
            last: None,
            temp: String::new(),
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
        self.last.replace(Rc::clone(&self.tag));
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
                    self.tag = Rc::new(RefCell::new(Tag::new(TagKind::Start)));

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
                    self.tag = Rc::new(RefCell::new(Tag::new(TagKind::End)));

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
                '\0' => self.tag.borrow_mut().append_name('\u{fffd}'),
                c => self.tag.borrow_mut().append_name(c.to_ascii_lowercase()),
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#rcdata-less-than-sign-state
            State::RawLessThanSign(kind) => match input {
                '/' => {
                    self.temp.drain(..);

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
                    self.tag = Rc::new(RefCell::new(Tag::new(TagKind::End)));

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
                    if self.tag.borrow().has_appropriate_end_tag(self.last.clone()) => self.state = State::BeforeAttributeName,
                '/' if self.tag.borrow().has_appropriate_end_tag(self.last.clone()) => self.state = State::SelfClosingStartTag,
                '>' if self.tag.borrow().has_appropriate_end_tag(self.last.clone()) => {
                    self.state = State::Data;

                    self.emit_tag();
                },
                c if c.is_ascii_alphabetic() => {
                    self.tag.borrow_mut().append_name(c.to_ascii_lowercase());

                    self.temp.push(c);
                }
                _ => {
                    self.sink
                        .emit([Token::CharacterToken('<'), Token::CharacterToken('/')]);

                    self.sink
                        .emit(self.temp.chars().map(|c| Token::CharacterToken(c)));

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
                    '\t' | '\n' | '\x0C' | ' ' | '/' | '>' => if self.temp.as_str() == "script" {
                        self.set_state_and_emit(State::ScriptDataEscaped(EscapeKind::DoubleEscaped), [Token::CharacterToken(input)]);
                    } else {
                        self.set_state_and_emit(State::ScriptDataEscaped(EscapeKind::Escaped), [Token::CharacterToken(input)]);
                    },
                    c if c.is_ascii_alphabetic() => {
                        self.temp.push(c.to_ascii_lowercase());

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
                    self.temp.drain(..);

                    self.state = State::ScriptDataEscapedEndTagOpen;
                },
                c if c.is_ascii_alphabetic() && kind == EscapeKind::Escaped => {
                    self.temp.drain(..);

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
                    self.tag = Rc::new(RefCell::new(Tag::new(TagKind::End)));

                    self.reconsume(input, State::ScriptDataEscapedEndTagName);
                },
                _ => {
                    self.sink.emit([Token::CharacterToken('<'), Token::CharacterToken('/')]);

                    self.reconsume(input, State::ScriptDataEscaped(EscapeKind::Escaped));
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-escaped-end-tag-name-state
            State::ScriptDataEscapedEndTagName => match input {
                '\t' | '\n' | '\x0C' | ' ' if self.tag.borrow().has_appropriate_end_tag(self.last.clone()) => self.state = State::BeforeAttributeName,
                '/' if self.tag.borrow().has_appropriate_end_tag(self.last.clone()) => self.state = State::SelfClosingStartTag,
                '>' if self.tag.borrow().has_appropriate_end_tag(self.last.clone()) => {
                    self.state = State::Data;

                    self.emit_tag();
                },
                c if c.is_ascii_alphabetic() => {
                    self.tag.borrow_mut().append_name(c.to_ascii_lowercase());

                    self.temp.push(c);
                },
                _ => {
                    self.sink.emit([Token::CharacterToken('<'), Token::CharacterToken('/')]);

                    self.sink.emit(self.temp.chars().map(|c| Token::CharacterToken(c)));

                    self.reconsume(input, State::ScriptDataEscaped(EscapeKind::Escaped));
                },
            },

            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-double-escape-end-state
            State::ScriptDataDoubleEscapeEnd => match input {
                '\t' | '\n' | '\x0C' | ' ' | '/' | '>' => if self.temp.as_str() == "script" {
                    self.set_state_and_emit(State::ScriptDataEscaped(EscapeKind::Escaped), [Token::CharacterToken(input)]);
                } else {
                    self.set_state_and_emit(State::ScriptDataEscaped(EscapeKind::DoubleEscaped), [Token::CharacterToken(input)]);
                },
                c if c.is_ascii_alphabetic() => {
                    self.temp.push(c.to_ascii_lowercase());

                    self.sink.emit([Token::CharacterToken(c)]);
                },
                _ => self.reconsume(input, State::ScriptDataEscaped(EscapeKind::DoubleEscaped)),
            },

            // TODO: 31 out of 71 states done, excluding character reference related states
            _ => unimplemented!(),
        }
    }
}


