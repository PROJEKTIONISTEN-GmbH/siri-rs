# siri-rs

[![crates.io](https://img.shields.io/crates/v/siri-rs.svg)](https://crates.io/crates/siri-rs)
[![docs.rs](https://docs.rs/siri-rs/badge.svg)](https://docs.rs/siri-rs)

CEN **SIRI** — *Service Interface for Real-time Information*, EN 15531 / CEN/TS 15531 —
in Rust. Read a producer's feed, or run one.

This release covers:

- the **framework**: the `<Siri>` envelope, service requests and deliveries,
  capabilities and discovery;
- the **publish/subscribe data hub**: the full subscription lifecycle, direct and
  fetched delivery, check-status and heartbeat, termination — as types *and* as
  the state machines that drive them;
- **Situation Exchange (SIRI-SX)**: incidents and disruptions, complete — validity
  and publication windows, sources, classifiers and reasons, everything a
  situation affects, its consequences, and the publishing actions it triggers.

It is **transport-agnostic**. The library owns the protocol; carrying bytes is
yours. That keeps it usable from any HTTP stack, from a message queue, or from a
test harness with no I/O at all.

## Conformance

Conformance is not a claim here, it is the test suite.

Every official SIRI v2.2 example document shipped with the standard is read into
these types, written back out, compared with the original element by element, and
validated against the official schemas. A document that loses content, invents
content, reorders content or fails validation fails the build. The same applies to
the German **VDV 736** profile messages.

Every enumeration is checked token by token against the `xsd:simpleType` it
transcribes, so a mistyped wire value is a test failure rather than a rejected
message in production.

`tests/fixtures/README.md` lists the documents covered and where they come from.

## Quick start

```rust
use siri_rs::framework::ServiceDeliveryPayload;
use siri_rs::{Siri, SiriPayload};

let xml = r#"<Siri xmlns="http://www.siri.org.uk/siri" version="2.0">
  <ServiceDelivery>
    <ResponseTimestamp>2026-03-04T08:15:00+01:00</ResponseTimestamp>
    <SituationExchangeDelivery>
      <ResponseTimestamp>2026-03-04T08:15:00+01:00</ResponseTimestamp>
      <Situations>
        <PtSituationElement>
          <CreationTime>2026-03-04T07:50:00+01:00</CreationTime>
          <SituationNumber>2026-0041</SituationNumber>
          <Source>
            <SourceType>feed</SourceType>
          </Source>
          <ValidityPeriod>
            <StartTime>2026-03-04T07:50:00+01:00</StartTime>
          </ValidityPeriod>
          <EquipmentReason>liftFailure</EquipmentReason>
          <Summary xml:lang="EN">Lift out of service at Central Station</Summary>
        </PtSituationElement>
      </Situations>
    </SituationExchangeDelivery>
  </ServiceDelivery>
</Siri>"#;

let message: Siri = siri_rs::from_str(xml)?;

if let SiriPayload::ServiceDelivery(delivery) = &message.payload {
    for payload in &delivery.deliveries {
        if let ServiceDeliveryPayload::SituationExchangeDelivery(sx) = payload {
            for situation in sx.pt_situations() {
                println!(
                    "{}: {}",
                    situation.situation_number,
                    situation.summary.first().map(|s| s.value.as_str()).unwrap_or("")
                );
            }
        }
    }
}
# Ok::<(), siri_rs::Error>(())
```

Both namespace bindings found in the wild are accepted — the SIRI namespace as the
document default, or bound to a prefix.

## Running an endpoint

`siri_rs::pubsub` implements both sides of the data hub as state machines that take
messages and a clock reading and hand back the messages that should go out:

```rust,no_run
use siri_rs::pubsub::{Producer, ProducerConfig, SituationSource};

# fn run<S: SituationSource>(source: S, now: chrono::DateTime<chrono::FixedOffset>) -> siri_rs::Result<()> {
let mut producer = Producer::new(ProducerConfig::new("MY-AGENCY"), source);

// ... on each incoming request:
# let incoming = String::new();
if let Some(reply) = producer.handle(&siri_rs::from_str(&incoming)?, now)? {
    let _body = siri_rs::to_string(&reply)?;
}

// ... and whenever the situations change:
producer.situations_changed();
for outbound in producer.poll(now) {
    let _body = siri_rs::to_string(&outbound.message)?;
}
# Ok(())
# }
```

`examples/sx_endpoint.rs` runs a complete subscription cycle — subscribe, notify,
fetch, deliver, terminate — between an in-process producer and consumer, and
prints every message exchanged. Run it with `cargo run --example sx_endpoint`.

## Over HTTP, with axum and reqwest

Because nothing here carries bytes, an endpoint is these state machines plus a
transport. Two examples show all of it:

- **`examples/sx_producer_axum.rs`** — a POST route hands what arrives to
  `Producer::handle` and answers with the reply it gets back; a timer drives
  `Producer::poll` and posts what has become due to the address each consumer named
  when it subscribed. It serves one route per delivery method, so pushing a delivery
  and announcing one for collection are both visible in a single run.
- **`examples/sx_consumer_reqwest.rs`** — subscribes, receives the data-ready
  notification on a route of its own, answers it with the acknowledgement
  `Consumer::handle` builds, fetches the data, prints the situations that arrive, and
  unsubscribes before it stops.

```sh
cargo run --example sx_producer_axum     # in one terminal
cargo run --example sx_consumer_reqwest  # in another
```

`tests/http_endpoint.rs` runs that wiring on a port the operating system picks and
drives full cycles through it — announced delivery, pushed delivery, a heartbeat and
a plain service request — validating every body that crosses the wire against the
official schemas and pinning the order of the exchange.

axum, reqwest and tokio are development dependencies, and stay that way. Which
transport to use is the application's decision; the examples make one so that the
seam is concrete.

## Running the tests

```sh
cargo test
```

The schema-validation tests shell out to `xmllint`, which is part of libxml2
(Debian/Ubuntu: `apt install libxml2-utils`, macOS: `brew install libxml2`). They
fail with a pointer to this note if it is missing rather than passing quietly.

## Roadmap

The framework and the publish/subscribe hub are complete and service-independent;
the remaining work is one functional service at a time, each on this foundation:

- Estimated Timetable (SIRI-ET) and Production Timetable (SIRI-PT)
- Stop Monitoring (SIRI-SM) and Stop Timetable (SIRI-ST)
- Vehicle Monitoring (SIRI-VM)
- Connection Monitoring (SIRI-CM) and Connection Timetable (SIRI-CT)
- General Message (SIRI-GM), Facility Monitoring (SIRI-FM), Control Actions (SIRI-CA)

Also open: structured `Extensions` payloads, which currently round-trip as opaque
content, and a fuller DATEX II binding for the road-situation records SIRI-SX can
embed.

## Licence

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT licence ([LICENSE-MIT](LICENSE-MIT))

at your option.

The XML schemas and example documents under `tests/fixtures/` are the published
CEN SIRI artefacts and remain © 2006–2026 CEN — see `tests/fixtures/README.md`.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 licence, shall be
dual licensed as above, without any additional terms or conditions.
