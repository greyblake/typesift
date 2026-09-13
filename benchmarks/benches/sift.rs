//! How much slower is `typesift` than a hand-written traversal?
//!
//! The group collects every `UserId` in the graph two ways:
//!
//! - `manual_iterator`, a chain of `impl Iterator`s driven from outside with `next`.
//! - `typesift`.
//!
//! `sample_company` is built once and borrowed through `black_box`, so what is measured is the
//! traversal rather than the setup.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use typesift::TypeSift;
use typesift_benchmarks::{UserId, sample_company};

/// Collect every id into a `Vec`. Both allocate.
fn collect(c: &mut Criterion) {
    let company = sample_company();

    let mut group = c.benchmark_group("collect");
    group.bench_function("manual_iterator", |b| {
        b.iter(|| black_box(&company).user_ids().collect::<Vec<&UserId>>());
    });
    group.bench_function("typesift", |b| {
        b.iter(|| black_box(&company).sift::<UserId>());
    });
    group.finish();
}

criterion_group!(benches, collect);
criterion_main!(benches);
