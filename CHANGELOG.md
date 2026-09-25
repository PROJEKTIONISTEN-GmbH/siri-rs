# Changelog

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
  borrowed.

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
