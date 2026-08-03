# siri-rs

[![crates.io](https://img.shields.io/crates/v/siri-rs.svg)](https://crates.io/crates/siri-rs)
[![docs.rs](https://docs.rs/siri-rs/badge.svg)](https://docs.rs/siri-rs)

CEN **SIRI** — *Service Interface for Real-time Information*, EN 15531 / CEN/TS 15531 —
in Rust. Read a producer's feed, or run one.

## What is in it

- the **framework**: the `<Siri>` envelope, service requests and deliveries,
  capabilities and discovery;
- the **publish/subscribe data hub**: the full subscription lifecycle, direct and
  fetched delivery, check-status and heartbeat, termination — as types *and* as
  the state machines that drive them, for any of the services below;
- **Production Timetable (SIRI-PT)**: the day's plan — dated journeys, their calls,
  and the interchanges planned around them;
- **Estimated Timetable (SIRI-ET)**: the same journeys as they are actually running
  — delays, cancellations, journeys added today, stops skipped;
- **Stop Timetable (SIRI-ST)**: the same plan seen from one stop, as timetabled
  visits;
- **Stop Monitoring (SIRI-SM)**: the departure board — what is due at a stop now,
  how late it is running, and why the board is empty when it is;
- **Vehicle Monitoring (SIRI-VM)**: where the vehicles are, how they are getting on,
  and which stops they have served and have still to serve;
- **Connection Timetable (SIRI-CT)** and **Connection Monitoring (SIRI-CM)**: the
  interchanges planned over a connection link, and how they are going — the feeder
  side reporting its arrivals, the distributor side deciding whether to wait;
- **General Message (SIRI-GM)**: free-form messages on named channels, for what the
  structured services have no field for;
- **Facility Monitoring (SIRI-FM)**: whether the lift, the ticket machine or the
  accessible toilet is working, and what that means for passengers who need it;
- **Situation Exchange (SIRI-SX)**: incidents and disruptions, complete — validity
  and publication windows, sources, classifiers and reasons, everything a
  situation affects, its consequences, and the publishing actions it triggers;
- **Control Actions (SIRI-CA)**: what a control room has decided to do about all
  that — a journey put on or taken off, a stop closed, an interchange held or
  dropped, a vehicle moved to other work, a message sent to a driver;
- the **journey model** these services share, down to train formations, occupancy
  and capacity, the facility model, and the GML polygon a flexible stop area may be
  drawn as.

It is **transport-agnostic**. The library owns the protocol; carrying bytes is
yours. That keeps it usable from any HTTP stack, from a message queue, or from a
test harness with no I/O at all.

## Install

```sh
cargo add siri-rs
```

## Quick start

Reading a feed: parse a delivery, walk the situations it carries.

```rust
use siri_rs::framework::ServiceDeliveryPayload;
use siri_rs::{Siri, SiriPayload};

fn main() -> siri_rs::Result<()> {
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

    Ok(())
}
```

Both namespace bindings found in the wild are accepted — the SIRI namespace as the
document default, or bound to a prefix.

## Running an endpoint

`siri_rs::pubsub` implements both sides of the data hub as state machines. They turn
incoming messages and a clock reading into the messages that should go out; they open
no socket and read no clock of their own, so the same code can be driven by a real
transport or by a test.

`examples/sx_endpoint.rs` runs a complete subscription cycle — subscribe, notify,
fetch, deliver, terminate — between an in-process producer and consumer, with no I/O
at all, and prints every message exchanged:

```sh
cargo run --example sx_endpoint
```

## Over HTTP, with axum and reqwest

Because nothing here carries bytes, an endpoint is these state machines plus a
transport. On the producer's side, a route hands what arrives to `Producer::handle`
and answers with the reply it gets back, while a timer asks `Producer::poll` what has
fallen due and posts it to the address each consumer named when it subscribed:

```rust
use chrono::{DateTime, FixedOffset};
use siri_rs::pubsub::{Producer, Service, Source};

/// The route. `None` is a message that needs no answer — an acknowledgement, say —
/// which HTTP reports as `204 No Content`.
fn answer<S: Source<Svc>, Svc: Service>(
    producer: &mut Producer<S, Svc>,
    body: &str,
    now: DateTime<FixedOffset>,
) -> siri_rs::Result<Option<String>> {
    let message = siri_rs::from_str(body)?;
    producer
        .handle(&message, now)?
        .map(|reply| siri_rs::to_string(&reply))
        .transpose()
}

/// The timer. Every message that has fallen due — a delivery, a data-ready
/// notification, a heartbeat — paired with the address to post it to. A consumer
/// that named no address cannot be reached, so nothing is sent to it.
fn due<S: Source<Svc>, Svc: Service>(
    producer: &mut Producer<S, Svc>,
    now: DateTime<FixedOffset>,
) -> siri_rs::Result<Vec<(String, String)>> {
    producer
        .poll(now)
        .into_iter()
        .filter_map(|outbound| Some((outbound.address?, outbound.message)))
        .map(|(address, message)| Ok((address.to_string(), siri_rs::to_string(&message)?)))
        .collect()
}
```

On the consumer's side, opening a subscription yields the body to post and the
`Consumer` that will interpret everything the producer sends back:

```rust
use chrono::{DateTime, Duration, FixedOffset};
use siri_rs::pubsub::{Consumer, SituationExchange};
use siri_rs::sx::SituationExchangeRequest;

fn subscribe(
    own_address: &str,
    now: DateTime<FixedOffset>,
) -> siri_rs::Result<(Consumer<SituationExchange>, String)> {
    let mut consumer = Consumer::<SituationExchange>::new("PASSENGER-APP")
        .at_address(own_address)
        .confirming_deliveries();

    let request = consumer.subscribe(
        "disruptions",
        now + Duration::hours(1),
        SituationExchangeRequest::new(now),
        now,
    );

    Ok((consumer, siri_rs::to_string(&request)?))
}
```

`Consumer::handle` then turns each incoming message into a `ConsumerEvent`: the
records a delivery carried, the acknowledgement and the fetch request a data-ready
notification calls for, a heartbeat, a subscription that ended.

Which service the two speak is decided once. A producer takes it from its source —
implementing `SituationSource`, `StopMonitoringSource`, `FacilityMonitoringSource`
or any of the others is what makes it a producer of that service — and a consumer is
told directly, as `Consumer::<EstimatedTimetable>::new(…)`. Everything else is the
same code.

Four pairs of examples run all of it against a real socket. Each producer serves a
route and a timer; each consumer prints what arrives and unsubscribes before it stops.
**`examples/sx_producer_axum.rs`** serves one route per delivery method, so pushing a
delivery and announcing one for collection are both visible in a single run, and
**`examples/sx_consumer_reqwest.rs`** follows the announced path all the way through.
**`examples/et_producer_axum.rs`** publishes a journey running late and one cancelled,
and lets the delay grow while **`examples/et_consumer_reqwest.rs`** is watching.
**`examples/vm_producer_axum.rs`** moves a vehicle every few seconds, which
**`examples/vm_consumer_reqwest.rs`** follows across the map.
**`examples/sm_producer_axum.rs`** keeps a departure board where one tram keeps losing
time, and **`examples/sm_consumer_reqwest.rs`** asks for it both ways round: first by
polling, the way a display redraws itself, then by subscribing and watching the delay
grow.

```sh
cargo run --example sx_producer_axum     # in one terminal…
cargo run --example sx_consumer_reqwest  # …and the other in another

cargo run --example et_producer_axum
cargo run --example et_consumer_reqwest

cargo run --example vm_producer_axum
cargo run --example vm_consumer_reqwest

cargo run --example sm_producer_axum
cargo run --example sm_consumer_reqwest
```

`tests/http_endpoint.rs`, `tests/http_journey_services.rs` and
`tests/http_stop_monitoring.rs` run that wiring on a port the operating system picks
and drive full cycles through it — announced delivery, pushed delivery, a heartbeat,
a plain service request — validating every body that crosses the wire against the
official schemas and pinning the order of the exchange. `tests/hub_services.rs` does
the same in process for every service the crate models, so none of them can quietly
stop fitting the hub.

axum, reqwest and tokio are development dependencies, and stay that way. Which
transport to use is the application's decision; the examples make one so that the
seam is concrete.

## Conformance

Conformance is not a claim here, it is the test suite.

Every official SIRI v2.2 example document shipped with the standard is read into
these types, written back out, compared with the original element by element, and
validated against the official schemas. A document that loses content, invents
content, reorders content or fails validation fails the build. The same applies to
the German **VDV 736** profile messages.

Every enumeration is checked token by token against the `xsd:simpleType` it
transcribes, so a mistyped wire value is a test failure rather than a rejected
message in production. Losing an enumeration from that check is itself a failure:
the count is compared with the number the crate declares.

One service is the exception, and says so: **Control Actions ships no example
documents at all**, so there is nothing to read back and compare. It is checked
against the schemas instead — a message is built for every control action the
schema allows and each is validated — which is a weaker guarantee than the other
services have. Example messages would make it the same guarantee.

`tests/fixtures/README.md` lists the documents covered and where they come from.

## Running the tests

```sh
cargo test
```

The schema-validation tests shell out to `xmllint`, which is part of libxml2
(Debian/Ubuntu: `apt install libxml2-utils`, macOS: `brew install libxml2`). They
fail with a pointer to this note if it is missing rather than passing quietly.

## Roadmap

Every CEN functional service is now modelled, so there is no list of services left
to work through. What remains open is narrower:

- structured `Extensions` payloads, which currently round-trip as opaque content;
- a fuller DATEX II binding for the road-situation records SIRI-SX can embed;
- Control Actions proved by round trip rather than by the schemas alone, once
  example messages for it exist.

## Licence

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT licence ([LICENSE-MIT](LICENSE-MIT))

at your option.

The XML schemas and example documents under `tests/fixtures/` are the published
CEN SIRI artefacts and remain © 2006–2026 CEN — see `tests/fixtures/README.md`.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 licence, shall be
dual licensed as above, without any additional terms or conditions.
