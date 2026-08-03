//! What it costs to read a SIRI document.
//!
//! The `read` group measures [`siri_rs::from_str`] on documents of growing size,
//! ending with the two bindings of the SIRI namespace: a document that binds it by
//! default is handed to the deserialiser borrowed, one that binds it to a prefix has
//! to be rewritten first.

// `criterion_group!` expands to a function that has nowhere to carry documentation,
// so the crate's `missing_docs` lint is lifted for the benchmarks.
#![allow(missing_docs)]

mod support;

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use siri_rs::sx::RoadSituationElement;
use siri_rs::Siri;

fn read(c: &mut Criterion) {
    let mut group = c.benchmark_group("read");

    for case in support::siri_documents() {
        group.throughput(case.bytes());
        group.bench_with_input(
            BenchmarkId::from_parameter(case.name),
            &case.xml,
            |b, xml| {
                b.iter(|| siri_rs::from_str::<Siri>(black_box(xml)).expect("the fixture reads"));
            },
        );
    }

    let prefixed = support::prefixed_situation();
    group.throughput(prefixed.bytes());
    group.bench_with_input(
        BenchmarkId::from_parameter(prefixed.name),
        &prefixed.xml,
        |b, xml| {
            b.iter(|| {
                siri_rs::from_str::<RoadSituationElement>(black_box(xml)).expect("the fixture reads")
            });
        },
    );

    group.finish();
}

criterion_group!(benches, read);
criterion_main!(benches);
