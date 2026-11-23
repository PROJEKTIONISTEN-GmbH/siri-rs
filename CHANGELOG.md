# Changelog

## 2.5.0 — 2026-11-23

Facility discovery, and projections written in the namespace the schema puts
them in. Everything here is additive in the API; one change of what is written
is listed under *Changed*.

### Added

- **Facility discovery: `FacilityRequest` and `FacilityDelivery`.** A consumer
  asks which facilities a producer serves — the lifts, ticket machines and
  toilets it can then ask Facility Monitoring about — narrowed by a rectangle or
  a topographic place (`FacilityRequest::scope`, `FacilityScope`), an operator
  and a line, with preferred languages and a `FacilityDetailLevel`
  (`enumerations::FacilityDetail`; an empty element reads as `normal`, the
  schema's default). The producer answers with `AnnotatedFacility` entries, a
  facility named by reference and, at more detail, described. Both are
  `SiriPayload` variants (`as_facility_request`, `as_facility_delivery`); the
  request's `Debug` output withholds its `AccountKey` like every other.
  `AnnotatedFacility::new` names a facility by reference alone. The standard
  publishes no example of this exchange; the tests read and write documents
  derived from the stop points discovery and facility monitoring examples and
  validate them against the schemas.

### Changed

- **Projections are written in the IFOPT namespace.** `LinkProjection`,
  `PointProjection` and `ZoneProjection` are IFOPT types, and the schema puts
  their content — `Features`, `Line`, `Boundary`, the points and their
  coordinates — in the `ifopt` namespace even where a SIRI element carries them,
  as `LinkProjectionToNextStopPoint` in a lines discovery or an affected route
  does; a stop place component carries the IFOPT elements themselves. Earlier
  releases wrote all of it in the SIRI namespace, and the schema rejected every
  document carrying a projection. Each projection now binds the `ifopt` prefix
  on the element it is written as and writes its content under it.

  Documents written the earlier way still read, strictly and leniently: the
  reader matches an element by its local name. A consumer that compared this
  crate's output as text, rather than as XML, sees the prefix and the
  declaration where there were none.

### Known limitations

- Two discovery services are not modelled yet: `InfoChannelRequest` and
  `ConnectionLinksRequest`, with their deliveries. A document carrying one is
  refused as an unknown message.
- An empty facility feature — `<Feature><AccessFacility/></Feature>`, which the
  schema reads as `unknown` — is not read as its default, for any of the sixteen
  feature families.

### Upgrading from 2.4

- A `match` on `SiriPayload` needs a wildcard arm already, the enum being
  non-exhaustive; one that routes the discovery messages should take
  `SiriPayload::FacilityRequest` and `SiriPayload::FacilityDelivery`.

## 2.4.0 — 2026-11-22

What this crate's own producer writes for an estimated timetable of no
journeys now reads back, leniently. Everything here is additive in the API; two
changes of behaviour of the lenient reader are listed under *Changed*.

### Added

- **The lenient reader reads an estimated timetable of no journeys.** The
  schema demands a journey in every `EstimatedJourneyVersionFrame`, so it has no
  form for "no journeys", and a producer asked for journeys it has none of
  writes a frame that holds none — this crate's own producer does, answering a
  request or a fetch. `lenient::from_str` took that frame for a fault: it left
  the frame out, and the delivery with it, and failed the document when that
  delivery was its only one. It now reads the frame as holding no journey, keeps
  the delivery, and reports the frame as a finding of a new kind,
  `Discarded::Nothing`: the element was read, nothing was left out.
  `Findings::read_as_empty` lists these findings; `len` and `by_reason` count
  them, under the reason the strict reader gives, ``missing field
  `EstimatedVehicleJourney` ``. The strict reader refuses such a frame as before,
  with the feature on or off.

### Changed

- **A frame whose every journey the lenient reader leaves out is kept**, holding
  none, and reported as above, after the findings for its journeys. The frame
  went too before, and the delivery with it — or the document, where the
  delivery was its only one.
- **`Findings::elements` lists the findings of `Discarded::Element` only.** It
  listed every finding that left out no unit, which now takes in the new kind.

### Maintenance

- The source is laid out by rustfmt, at the 120 columns it was written to
  (`rustfmt.toml`); the publication checks among the tests hold it to that.
- A CI workflow runs the tests with and without the `lenient` feature, rustfmt,
  clippy in both variants, rustdoc with warnings denied, and checks the library
  against the minimum supported Rust, 1.75.

### Upgrading from 2.3

- A `match` on `Discarded` needs a wildcard arm already, the enum being
  non-exhaustive; one that takes every finding that is not a unit for an element
  left out should match `Discarded::Nothing` for a frame read as holding none.

## 2.3.0 — 2026-11-21

A review of 2.2.0 found that what arrives was not always handed on, that the
lenient reader could read a value no producer sent, and that a document outside
SIRI's namespace or outside XML was read as SIRI. Everything here is additive in
the API; four changes of behaviour are listed under *Changed*, with what to do.

### Added

- **`ConsumerEvent::DeliveredWhole`.** A delivery carries more than the records
  the hub publishes: the withdrawal of a record delivered earlier
  (`MonitoredStopVisitCancellation`, `VehicleActivityCancellation`,
  `TimetabledStopVisitCancellation`, `GeneralMessageCancellation`, the feeder
  arrival cancellations), the notices and service exceptions of a departure
  board, road situations, the interchanges of an estimated timetable, a note on
  the delivery, and the situations a delivery is accompanied by. `Delivered`
  passed on the records alone, so a consumer of incremental updates kept
  showing a departure its producer had withdrawn. A message that holds any of
  these now arrives as `DeliveredWhole`, with the records `Delivered` would carry
  and the `ServiceDelivery` whole.
- **`ConsumerEvent::Unavailable`.** A heartbeat or a status answer whose `Status`
  is false arrived as `Alive`; it now arrives as `Unavailable`, with the error
  condition the producer gave — `ServiceNotAvailableError` and its expected
  restart time, or `OtherError`.
- **`Service::delivery_of`, `Service::holds_more_than_items` and
  `Service::CAN_DELIVER_NOTHING`.** The trait is sealed, so these are not breaking.
  `delivery_of` picks a service's own deliveries out of a `ServiceDelivery`.
- **`ServiceDeliveryPayload::IncludedSituationExchangeDelivery`.** Schema 2.2
  lets situations accompany a delivery of another service, ahead of it. Such a
  document was refused whole by the strict reader and lost its situations in the
  lenient one; it now reads, is written with the situations ahead of the other
  deliveries wherever they stand in the list, and reads leniently situation by
  situation. A derived example document covers it.
- **`Error::UnexpectedNamespace`, `Error::ContentAfterRoot` and
  `Error::IllegalCharacter`**, for the refusals below. `Error` is
  non-exhaustive, so these are not breaking either.

### Changed

- **A root element outside the SIRI namespace is refused**, whether in another
  namespace or in none (`Error::UnexpectedNamespace`); it was read as SIRI.
  A producer that writes `<Siri>` without `xmlns="http://www.siri.org.uk/siri"`
  writes no SIRI document, and is now told so.
- **What XML does not allow is refused.** A control character other than tab,
  line feed and carriage return, U+FFFE or U+FFFF, written as itself or as a
  character reference, fails reading and writing alike
  (`Error::IllegalCharacter`): such a value was written, and every conforming
  reader would have refused the document. Anything after the root element but
  comments, processing instructions and whitespace — a second root, text — fails
  reading (`Error::ContentAfterRoot`); it was ignored.
- **The lenient reader leaves out no element that would then read as a value.**
  An element the schema gives a default — `Cancellation`, `Monitored`,
  `VehicleAtStop` and some two hundred others — means that default when absent,
  so leaving one out alone made `<Cancellation>True</Cancellation>` read as a
  journey that runs. The element around it is left out instead, by the same
  rules as for any other: here the journey. The finding still names the element
  at fault.
- **An Estimated Timetable subscription is sent nothing while its source holds
  no journey** — no delivery, no data-ready notification — since the schema has
  no form for such a delivery and this crate's own reader refused the one the
  producer wrote. A request or a fetch is still answered with the empty frame;
  there is no valid answer to give.

### Fixed

- An empty submode element of the network a situation's context names —
  `<BusSubmode/>` and the seven others — read as an unrecognised empty token; it
  reads as `unknown`, the value the schema declares. The test that holds every
  field to the declared values missed these eight because their types are
  written with their path; it no longer does.
- A timestamp without a time-zone offset, or at `24:00:00`, is still refused, and
  now the error says why instead of chrono's "premature end of input" and "input
  is out of range".
- `Debug` output of the thirteen messages that carry an `AccountKey` shows that
  a key is present, never the key itself. A request logged with `{:?}` no longer
  hands the secret to whoever reads the log.

### Deprecated

- `sx::affects::Network`, a second transcription of the schema type
  `sx::delivery::Network` transcribes, held by no field of this crate.

### Known limitations

- `Facilities` of `TrainElement`, `Train` and `CompoundTrain` is not modelled
  and is dropped when read; the structs are exhaustive, so the field waits for
  the next major release.
- Below the root, an element is matched by its local name: a foreign element
  that shares a SIRI element's name where the schema expects that element is
  read as it, and the order of the schema's sequences is not enforced.

### Upgrading from 2.2

- A `match` on `ConsumerEvent` that handles `Delivered` should handle
  `DeliveredWhole` too: `Delivered { items, outcome, reply }` and
  `DeliveredWhole { items, outcome, reply, .. }` bind the same names, so one
  or-pattern serves both where the extra records do not matter. A monitor that
  watches `Alive` should treat `Unavailable` as the outage it is.
- A program that reads documents without the SIRI namespace declaration has to
  have its producer declare it.

## 2.2.0 — 2026-09-26

### Added

- A lenient reader, behind the `lenient` feature: `siri_rs::lenient::from_str`
  reads a document whose strict reading fails inside a functional service
  delivery for what reads. The element at fault is left out where the schema
  makes it optional; where the schema demands it, the smallest enclosing
  element the document reads without goes, up to the unit — a situation, a
  journey, a stop visit, a vehicle, a facility, a message, or the delivery
  itself while another remains. A list of records the reader finds interrupted
  by another element keeps its first run. Every element left out is a
  `Finding`, with the element's path in the form the reader's errors use, the
  reason the reader gave, and what went for it, a unit named by its
  `SituationNumber`, `DatedVehicleJourneyRef`, `ItemIdentifier` or the like;
  `Findings` counts them, groups them by reason and lists the units. The
  envelope, the only delivery, and everything wrong with the document rather
  than in it — XML that is not well-formed, a root that is not `<Siri>` — fail
  as they do strictly. Nothing is guessed and nothing is put in a fault's
  place. A document the strict reader accepts reads leniently as it does
  strictly, with no findings and at the same cost; one that does not is read
  again in pieces, each record on its own, so a delivery with a hundred faults
  costs a few readings of its size rather than a hundred.
- The feature adds the reader and nothing else: `siri_rs::from_str` reads
  exactly as before with the feature on or off, and no dependency comes with it.
  docs.rs builds the documentation with the feature, and the `read` benchmark
  measures the lenient reader next to the strict one.

## 2.1.0 — 2026-09-25

### Added

- An element the schema gives a default value reads as that value when it is
  written empty. XML Schema declares a default for 279 elements — booleans,
  enumerations, language tags, durations and a few numbers — and an element so
  declared that is written with no content is valid and means the declared
  value: `<Monitored/>` is `true`, `<Severity/>` is `normal`, `<Language/>` is
  `en`. Such an element failed the whole document with
  `invalid type: string "", expected a boolean`; it now reads as the declared
  value, in either empty spelling. The declarations are read out of the bundled
  schemas by the test suite and kept as a table the crate reads through, every
  field that transcribes one is checked against the table, and every value the
  crate can read is read out of an empty element in a real document. The value
  is written back in full, `<Monitored>true</Monitored>`, which the schema holds
  equal to the empty element; whether the document spelled the value out is not
  kept. `Allow` alone keeps its wording, through `DefaultedBoolean`, as before.
  Where an element is declared with a default in one place and without one in
  another, every field of that name reads the default: an empty element where
  none is declared is not valid, so no valid document reads differently.
- Eighteen derived example documents under `tests/fixtures/derived/`, carrying the
  declared elements the official examples do not; three of them are Control
  Actions messages, of which the standard publishes no example.

### Fixed

- A text of whitespace alone — `<Summary>   </Summary>`, which the schema
  accepts — was read as an empty text and written back as `<Summary/>`, which
  it refuses. It is read as the whitespace it is and written back the same, in
  every text and in open content. The whitespace that indents one element
  inside another is not text and is not read as any. A document holding such an
  element is rewritten once before it is read, like one that binds the SIRI
  namespace to a prefix; every other document is still handed to the reader
  borrowed. Looking a document over for such an element costs a few per cent
  of the time it takes to read it.

### Known limitations

- An empty space-separated list — `<MonitoringError/>`, `<FareClasses/>` — is
  read as an empty list and left out when written. A `Vec` cannot tell an
  absent element from an empty one, and telling them apart changes the field's
  type, which waits for a major release. The meaning is the same either way.
- `HasDriverMessages` and `HasVehicleDetectings` of the Control Actions
  capabilities are transcribed in the response features, where the schema
  declares them in the request policy; a document carrying them there does not
  validate. Moving them changes a public struct and waits for a major release.

## 2.0.1 — 2026-09-25

### Fixed

- Empty text / posList elements no longer fail deserialisation. An element of
  `NaturalLanguageString`, `DefaultedText` or `PosList` with no content —
  `<Summary/>`, `<Summary></Summary>`, `<gml:posList/>` — failed the whole
  document with "missing field `$text`"; it is now read as an empty `value`, and
  the rest of the document with it. An empty position list is valid GML. An
  empty text is not valid SIRI, which asks for at least one character, but real
  feeds carry one; it is read, and written back as it stands.

## 2.0.0 — 2026-09-14

A review of 1.0.0 read the whole crate against the schemas and against what a
producer meets at an open port. Everything it found is fixed here, and the fixes
change the contract: this is what breaks, what is new, and what to do about it.

### Breaking

- **Every public enum is `#[non_exhaustive]`.** A `match` on one needs a wildcard
  arm. The standard adds variants with every revision; from now on that is a minor
  release, not a major one.
- **The enumerations keep unknown tokens.** Each of the 112 schema enumerations
  has an `Unrecognised(String)` variant that holds a token the transcribed schema
  release does not list, and writes it back unchanged. A document carrying one is
  read whole where 1.0.0 refused it. As a consequence these types are no longer
  `Copy`: clone them, or borrow. `as_str` takes `&self` and returns `&str`;
  `FromStr` cannot fail, and `from_token` says so in its signature.
- **`Error::Deserialize` is a struct variant** carrying `path`, `offset` and
  `source`. Its message names the element that could not be read —
  `ServiceDelivery.StopMonitoringDelivery[0].MonitoredStopVisit[1]` — and how far
  into the document the reader had got.
- **Two error variants replace two misuses.** A message the producer cannot act
  on is `Error::UnexpectedMessage`, and a message lacking an element the answer
  needs is `Error::MissingElement`; `UnexpectedRoot` and `InvalidValue` mean only
  what their names say.
- **`Siri` has no `extensions` field.** `<Extensions>` is one of the schema's
  alternatives under `<Siri>`, not a trailer after a message, and is now
  `SiriPayload::Extensions`. A document of extensions alone reads; a message with
  extensions after it was never valid and can no longer be written.
- **`AnyContent` holds its content in order.** The `text` and `children` fields
  are replaced by `content: Vec<Node>`, where a `Node` is a run of character data
  or a child element, in document order; mixed content survives. `character_data`
  joins the text, `children` and `children_named` walk the elements.
- **`ConsumerEvent::Delivered` carries an `outcome`** — the message's status,
  error condition and `MoreData`, and the status, error condition and subscription
  of each delivery of the consumer's service in it. A delivery of no records and
  a delivery that failed are no longer the same event.
- **`Subscription` and `ProducerConfig` have no public fields.** Read a
  subscription through its accessors; build a configuration with `new` and the
  `with_…` methods, which now include `with_shortest_possible_cycle` and
  `with_message_id_prefix`.
- **`DataSupplyRequest::new` takes no notification.** Set `notification_ref`
  when the fetch answers one particular announcement; the element is left out
  otherwise, where 1.0.0 wrote it empty.
- **`Service` is sealed.** It was documented as not for applications to
  implement; now it cannot be. The trait also gained `outcome_of`.
- **`Duration::parse` rejects what the schema does not allow**: designators out
  of order or repeated, and a fraction on anything but the seconds or without
  digits on both sides of the point. `to_std` returns `None` instead of
  panicking for a length `std::time::Duration` cannot hold.
- **A producer refuses per entry.** A subscription request whose entry the
  producer cannot serve is answered with a refused status for that entry rather
  than an `Err` for the request. A termination request naming a subscription the
  subscriber does not hold is answered with `UnknownSubscriptionError` rather
  than a confirmation.

### Fixed

- A document nested a few thousand elements deep inside open content
  overflowed the reading thread's stack and aborted the process. Open content
  may nest at most `AnyContent::MAX_DEPTH` (64) levels; deeper is an `Err`.
- Two subscribers choosing the same subscription identifier lost one
  subscription: subscriptions are keyed by subscriber and identifier, as the
  standard scopes them.
- A heartbeat reached only the first subscription held; it now reaches every
  endpoint holding one.
- A request mixing services failed as a whole after its first entry had been
  held and delivered to; every entry is now judged before any is held.
- A fetch that found nothing waiting was answered with a `ServiceDelivery` the
  schema rejects; it is answered with a delivery of the service carrying no
  records. Estimated Timetable has no valid form for that — a version frame and a
  journey in it are mandatory — which a test now records rather than hides.
- The `gtfs-realtime` keyword, which described a different standard, is gone.

### Added

- `AnyContent::MAX_DEPTH`, `AnyContent::character_data`, `AnyContent::children`,
  `Node`.
- `SubscriptionRequestPayload::subscription_identifier`.
- `TerminationResponseStatus::refused`.
- `SiriPayload::as_extensions`.
- `DeliveryOutcome`, `FunctionalDeliveryOutcome`, `DeliveryOutcome::succeeded`.
- One runtime dependency, `serde_path_to_error`, which the crate already carried
  at one remove through its development dependencies.

### Upgrading from 1.0.0

1. Add a `_ =>` arm to every `match` on a crate enum, and decide what to do with
   an `Unrecognised` token where you match on an enumeration.
2. Replace `.copied()` on enumeration values with `.clone()` or a borrow.
3. Read `Error::Deserialize { path, offset, source }` where you read
   `Error::Deserialize(e)`; match `UnexpectedMessage` and `MissingElement` where
   you matched producer errors.
4. Replace `siri.extensions` with `SiriPayload::Extensions`.
5. Replace `content.text` with `content.character_data()` and
   `content.children` with `content.children()` or `content.content`.
6. Destructure `ConsumerEvent::Delivered { items, outcome, reply }` and look at
   `outcome.succeeded()` before treating an empty `items` as "nothing new".
7. Read subscriptions through `subscription_ref()`, `state()` and the other
   accessors; configure producers through `ProducerConfig::new(..).with_…`.
