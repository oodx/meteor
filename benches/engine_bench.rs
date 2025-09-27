use hub::criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use meteor::MeteorEngine;

fn bench_engine_set(c: &mut Criterion) {
    c.bench_function("engine: set operation", |b| {
        let mut engine = MeteorEngine::new();
        let mut counter = 0;
        b.iter(|| {
            let path = format!("app:ui:button{}", counter);
            engine.set(black_box(&path), black_box("click")).unwrap();
            counter += 1;
        })
    });
}

fn bench_engine_get_existing(c: &mut Criterion) {
    let mut engine = MeteorEngine::new();
    engine.set("app:ui:button", "click").unwrap();

    c.bench_function("engine: get existing", |b| {
        b.iter(|| {
            engine.get(black_box("app:ui:button"))
        })
    });
}

fn bench_engine_get_missing(c: &mut Criterion) {
    let engine = MeteorEngine::new();

    c.bench_function("engine: get missing", |b| {
        b.iter(|| {
            engine.get(black_box("app:ui:nonexistent"))
        })
    });
}

fn bench_engine_delete(c: &mut Criterion) {
    c.bench_function("engine: delete operation", |b| {
        b.iter_batched(
            || {
                let mut engine = MeteorEngine::new();
                engine.set("app:ui:button", "click").unwrap();
                engine
            },
            |mut engine| {
                engine.delete(black_box("app:ui:button")).unwrap();
            },
            hub::criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_engine_contexts(c: &mut Criterion) {
    let mut engine = MeteorEngine::new();
    engine.set("app:ui:button", "click").unwrap();
    engine.set("user:settings:theme", "dark").unwrap();
    engine.set("system:config:debug", "true").unwrap();

    c.bench_function("engine: list contexts", |b| {
        b.iter(|| {
            engine.contexts()
        })
    });
}

fn bench_engine_namespaces(c: &mut Criterion) {
    let mut engine = MeteorEngine::new();
    engine.set("app:ui:button", "click").unwrap();
    engine.set("app:settings:theme", "dark").unwrap();
    engine.set("app:config:debug", "true").unwrap();

    c.bench_function("engine: list namespaces", |b| {
        b.iter(|| {
            engine.namespaces_in_context(black_box("app"))
        })
    });
}

fn bench_engine_bulk_insert(c: &mut Criterion) {
    c.bench_function("engine: bulk insert 100 items", |b| {
        b.iter(|| {
            let mut engine = MeteorEngine::new();
            for i in 0..100 {
                let path = format!("app:ui:item{}", i);
                engine.set(&path, "value").unwrap();
            }
        })
    });
}

criterion_group!(
    engine_benches,
    bench_engine_set,
    bench_engine_get_existing,
    bench_engine_get_missing,
    bench_engine_delete,
    bench_engine_contexts,
    bench_engine_namespaces,
    bench_engine_bulk_insert
);
criterion_main!(engine_benches);