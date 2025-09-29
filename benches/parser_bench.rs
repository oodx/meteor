use hub::criterion::{criterion_group, criterion_main, Criterion};
use meteor::parser::meteor_stream::MeteorStreamParser;
use meteor::parser::token_stream::TokenStreamParser;
use meteor::{MeteorEngine, Token};
use std::hint::black_box;

fn bench_token_stream_simple(c: &mut Criterion) {
    c.bench_function("token_stream: simple parsing", |b| {
        b.iter(|| {
            let mut engine = MeteorEngine::new();
            let input = black_box("button=click;theme=dark;user=admin");
            TokenStreamParser::process(&mut engine, input)
        })
    });
}

fn bench_token_stream_complex(c: &mut Criterion) {
    c.bench_function("token_stream: complex with folding", |b| {
        b.iter(|| {
            let mut engine = MeteorEngine::new();
            let input = black_box("ctx=app;ns=ui;button=click;ns=settings;theme=dark;color=blue");
            TokenStreamParser::process(&mut engine, input)
        })
    });
}

fn bench_meteor_stream_explicit(c: &mut Criterion) {
    c.bench_function("meteor_stream: explicit addressing", |b| {
        b.iter(|| {
            let mut engine = MeteorEngine::new();
            let input = black_box("app:ui:button=click :;: user:settings:theme=dark");
            MeteorStreamParser::process(&mut engine, input)
        })
    });
}

fn bench_token_parse(c: &mut Criterion) {
    c.bench_function("token: parse simple", |b| {
        b.iter(|| {
            let input = black_box("button=click");
            Token::parse(input)
        })
    });
}

fn bench_token_parse_multiple(c: &mut Criterion) {
    c.bench_function("token: parse multiple (10 tokens)", |b| {
        b.iter(|| {
            let input = black_box("a=1;b=2;c=3;d=4;e=5;f=6;g=7;h=8;i=9;j=10");
            Token::parse(input)
        })
    });
}

criterion_group!(
    parser_benches,
    bench_token_stream_simple,
    bench_token_stream_complex,
    bench_meteor_stream_explicit,
    bench_token_parse,
    bench_token_parse_multiple
);
criterion_main!(parser_benches);
