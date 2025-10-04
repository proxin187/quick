use quick_html::tokenizer::{Tokenizer, Token, TokenSink};

struct Sink;

impl TokenSink for Sink {
    fn process(&mut self, token: Token) {
        println!("token: {:?}", token);
    }

    fn eof(&self) {
        println!("end of file");
    }
}

fn main() {
    let string = std::fs::read_to_string("example.html").unwrap();
    let mut sink = Sink;

    Tokenizer::new(&mut sink, string.chars())
        .wait();
}


