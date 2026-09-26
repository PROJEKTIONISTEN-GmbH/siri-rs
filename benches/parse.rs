//! What it costs to read a SIRI document.
//!
//! The `read` group measures [`siri_rs::from_str`] on documents of growing size,
//! ending with the two bindings of the SIRI namespace: a document that binds it by
//! default is handed to the deserialiser borrowed, one that binds it to a prefix has
//! to be rewritten first. With the `lenient` feature, the `read-lenient` group
//! measures the lenient reader on the same intact documents — the cost of asking
//! for it when nothing is wrong — and on one that has a unit to leave out.

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

#[cfg(feature = "lenient")]
fn read_leniently(c: &mut Criterion) {
    let mut group = c.benchmark_group("read-lenient");

    for case in support::siri_documents() {
        group.throughput(case.bytes());
        group.bench_with_input(BenchmarkId::new("intact", case.name), &case.xml, |b, xml| {
            b.iter(|| siri_rs::lenient::from_str(black_box(xml)).expect("the fixture reads"));
        });
    }

    let damaged = support::one_visit_lost();
    group.throughput(damaged.bytes());
    group.bench_with_input(BenchmarkId::new("one unit lost", damaged.name), &damaged.xml, |b, xml| {
        b.iter(|| {
            let (_, findings) = siri_rs::lenient::from_str(black_box(xml)).expect("the rest reads");
            assert_eq!(findings.len(), 1);
        });
    });

    group.finish();
}

#[cfg(feature = "lenient")]
criterion_group!(benches, read, read_leniently);
#[cfg(not(feature = "lenient"))]
criterion_group!(benches, read);
criterion_main!(benches);
