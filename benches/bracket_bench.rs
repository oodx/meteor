use hub::criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use meteor::{TokenKey, BracketNotation};

fn bench_bracket_tokenkey_simple(c: &mut Criterion) {
    c.bench_function("bracket: tokenkey simple index", |b| {
        b.iter(|| {
            TokenKey::new(black_box("list[0]"))
        })
    });
}

fn bench_bracket_tokenkey_multidim(c: &mut Criterion) {
    c.bench_function("bracket: tokenkey multidimensional", |b| {
        b.iter(|| {
            TokenKey::new(black_box("grid[5,10]"))
        })
    });
}

fn bench_bracket_tokenkey_nested(c: &mut Criterion) {
    c.bench_function("bracket: tokenkey nested", |b| {
        b.iter(|| {
            TokenKey::new(black_box("data[users][0][name]"))
        })
    });
}

fn bench_bracket_tokenkey_no_brackets(c: &mut Criterion) {
    c.bench_function("bracket: tokenkey no brackets", |b| {
        b.iter(|| {
            TokenKey::new(black_box("simple_key"))
        })
    });
}

fn bench_bracket_has_brackets(c: &mut Criterion) {
    let key_with = TokenKey::new("list[0]");
    let key_without = TokenKey::new("simple");

    c.bench_function("bracket: has_brackets check", |b| {
        b.iter(|| {
            black_box(&key_with).has_brackets() || black_box(&key_without).has_brackets()
        })
    });
}

fn bench_tokenkey_new(c: &mut Criterion) {
    c.bench_function("tokenkey: new with transformation", |b| {
        b.iter(|| {
            TokenKey::new(black_box("list[0]"))
        })
    });
}

fn bench_tokenkey_display(c: &mut Criterion) {
    let key = TokenKey::new("list[0]");
    c.bench_function("tokenkey: display (flat)", |b| {
        b.iter(|| {
            format!("{}", black_box(&key))
        })
    });
}

fn bench_tokenkey_to_bracket(c: &mut Criterion) {
    let key = TokenKey::new("list[0]");
    c.bench_function("tokenkey: to_bracket", |b| {
        b.iter(|| {
            black_box(&key).to_bracket()
        })
    });
}

criterion_group!(
    bracket_benches,
    bench_bracket_tokenkey_simple,
    bench_bracket_tokenkey_multidim,
    bench_bracket_tokenkey_nested,
    bench_bracket_tokenkey_no_brackets,
    bench_bracket_has_brackets,
    bench_tokenkey_display,
    bench_tokenkey_to_bracket
);
criterion_main!(bracket_benches);