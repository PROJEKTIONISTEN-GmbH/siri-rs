//! What it costs to write a SIRI document.
//!
//! Every case is one of the read fixtures written back out, so the values are the
//! ones the crate really produces. Throughput counts the bytes written rather than
//! the bytes of the fixture they came from.

// `criterion_group!` expands to a function that has nowhere to carry documentation,
// so the crate's `missing_docs` lint is lifted for the benchmarks.
#![allow(missing_docs)]

mod support;

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use siri_rs::sx::RoadSituationElement;
use siri_rs::{Siri, SiriRoot};

fn write(c: &mut Criterion) {
    let mut group = c.benchmark_group("write");

    for case in support::siri_documents() {
        let value: Siri = siri_rs::from_str(&case.xml).expect("the fixture reads");
        group.throughput(written_bytes(&value));
        group.bench_with_input(BenchmarkId::from_parameter(case.name), &value, |b, value| {
            b.iter(|| siri_rs::to_string(black_box(value)).expect("the value writes"));
        });
    }

    let prefixed = support::prefixed_situation();
    let situation: RoadSituationElement = siri_rs::from_str(&prefixed.xml).expect("the fixture reads");
    group.throughput(written_bytes(&situation));
    group.bench_with_input(
        BenchmarkId::from_parameter(prefixed.name),
        &situation,
        |b, value| {
            b.iter(|| siri_rs::to_string(black_box(value)).expect("the value writes"));
        },
    );

    group.finish();
}

/// The indented layout, which the conformance suite and the examples write.
fn write_pretty(c: &mut Criterion) {
    let mut group = c.benchmark_group("write-pretty");

    for case in support::siri_documents() {
        let value: Siri = siri_rs::from_str(&case.xml).expect("the fixture reads");
        group.throughput(Throughput::Bytes(
            siri_rs::to_string_pretty(&value).expect("the value writes").len() as u64,
        ));
        group.bench_with_input(BenchmarkId::from_parameter(case.name), &value, |b, value| {
            b.iter(|| siri_rs::to_string_pretty(black_box(value)).expect("the value writes"));
        });
    }

    group.finish();
}

fn written_bytes<T: SiriRoot>(value: &T) -> Throughput {
    Throughput::Bytes(siri_rs::to_string(value).expect("the value writes").len() as u64)
}

criterion_group!(benches, write, write_pretty);
criterion_main!(benches);
