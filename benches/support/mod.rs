//! The documents the benchmarks run on.
//!
//! They are the official example documents the conformance suite already reads, so
//! the numbers describe work this crate really does rather than work invented for
//! the occasion.
//!
//! Cargo compiles this module separately into each benchmark binary, and no single
//! binary uses all of it.
#![allow(dead_code)]

use std::path::Path;

use criterion::Throughput;

/// One benchmark input.
pub struct Case {
    /// The document's path under `tests/fixtures/xml`, which names the benchmark.
    pub name: &'static str,
    /// The document itself.
    pub xml: String,
}

impl Case {
    /// How many bytes the document is, for the throughput figure.
    pub fn bytes(&self) -> Throughput {
        Throughput::Bytes(self.xml.len() as u64)
    }
}

/// Reads one example document.
pub fn case(name: &'static str) -> Case {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/xml")
        .join(name);
    let xml = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    Case { name, xml }
}

/// The `<Siri>`-rooted documents the reading and writing benchmarks run on: the
/// shortest message the framework has, a real-time delivery from each of the three
/// services that carry many calls or vehicles, and the two largest situation
/// exchanges in the fixtures.
pub fn siri_documents() -> Vec<Case> {
    [
        "framework/exa_checkStatus_request.xml",
        "et/ext_estimatedTimetable_response.xml",
        "vm/exv_vehicleMonitoring_response.xml",
        "sm/exs_stopMonitoring_response_complex.xml",
        "sx/exx_situationExchange_response.xml",
        "sx/vdv736/SX_1022_main_message.xml",
    ]
    .into_iter()
    .map(case)
    .collect()
}

/// The largest fixture that binds the SIRI namespace to a prefix.
///
/// Reading it is the path that has to rewrite the document before deserialising,
/// rather than hand it on borrowed.
pub fn prefixed_situation() -> Case {
    case("sx/exx_situationExchange_road.xml")
}

/// A document bound to the SIRI namespace by default, which reading hands on
/// borrowed.
pub fn default_bound_situation() -> Case {
    case("sx/exx_situationExchange_response.xml")
}
