use criterion::{criterion_main, criterion_group, Criterion};
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

fn benchmarks(c: &mut Criterion) {
    let string = std::fs::read_to_string("benches/data/small-fragment.html").unwrap();

    c.bench_function("small-fragment", move |b| {
        b.iter(|| {
            let mut sink = Sink;
            let mut tokenizer = Tokenizer::new(&mut sink, string.chars());

            while tokenizer.step() {}
        });
    });
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);

