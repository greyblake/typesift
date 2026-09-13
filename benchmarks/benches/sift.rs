//! How much slower is `typesift` than a hand-written traversal?
//!
//! The group collects every `UserId` in the graph three ways:
//!
//! - `manual_iterator`, a chain of `impl Iterator`s driven from outside with `next`.
//! - `manual_collector`, the same walk pushing straight into a `Vec`. Monomorphic and eager, so
//!   it is the floor: the fastest this is likely to get by hand.
//! - `typesift`.
//!
//! All three start from an empty `Vec`, so none of them gets a head start on allocation.
//! `sample_company` is built once and borrowed through `black_box`, so what is measured is the
//! traversal rather than the setup.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use typesift::TypeSift;
use typesift_benchmarks::{UserId, sample_company};

/// Collect every id into a `Vec`. All three allocate.
fn collect(c: &mut Criterion) {
    let company = sample_company();

    let mut group = c.benchmark_group("collect");
    group.bench_function("manual_iterator", |b| {
        b.iter(|| black_box(&company).user_ids().collect::<Vec<&UserId>>());
    });
    group.bench_function("manual_collector", |b| {
        b.iter(|| {
            let mut ids: Vec<&UserId> = Vec::new();
            black_box(&company).collect_user_ids(&mut ids);
            ids
        });
    });
    group.bench_function("typesift", |b| {
        b.iter(|| black_box(&company).sift::<UserId>());
    });
    group.finish();
}

criterion_group!(benches, collect);
criterion_main!(benches);
