# Derived fixtures

Documents in this directory are **not** published CEN material — everything under
`../xml` is, and stays untouched. These are derived from the schemas and from the
official examples to cover content the official examples leave uncovered, and they
are held to the same bar: read into the crate's types, written back out, compared
element by element, and validated against `../xsd/siri.xsd`.

Each one says below what it is derived from and why it exists.

## `framework/exa_checkStatus_request_extensions.xml`

The official `xml/framework/exa_checkStatus_request.xml`, with a payload inside its
`<Extensions>` element.

Why derived: every official example that carries `<Extensions>` at all carries it
empty (`<Extensions/>`), so nothing published exercises the content the schema
admits there — `ExtensionsStructure` is an `xsd:any` wildcard with
`processContents="lax"`. The payload is invented for the occasion and says so: its
namespaces are under `example.org`, reserved for documentation by RFC 2606. It
covers what an extension payload can be made of — nested elements, attributes, a
repeated element name, a subtree qualified by a prefix, and a subtree qualified by
a default namespace declaration.

## `sx/exx_situationExchange_road_datex.xml`

Modelled on the official `xml/sx/exx_situationExchange_road.xml`, trimmed to the
elements the schema requires, and carrying a DATEX II payload in the `<Road>`
element that `AffectedRoadStructure` types as a DATEX
`RoadsideReferencePointLinear`.

Why derived: the official road example does carry a DATEX II record, but inside an
XML comment — so no published document actually exchanges DATEX content through
SIRI. The payload here is the schema's own content model for that type, filled with
the road and reference-point identifiers the commented-out example uses.

Which DATEX slot: `RoadsideReferencePointLinear` is a concrete DATEX type, so the
element needs no `xsi:type` to name a subtype. The commented-out record in the
official example sits in `<SituationRecord>`, whose DATEX type is abstract and
therefore needs `xsi:type` — an attribute in the `xsi` namespace, and a prefix on
an attribute inside opaque content is the one thing the read path cannot carry
back (`types::AnyContent` documents why). That document is therefore not derivable
into a round-tripping fixture yet.

## Elements the schema gives a default value

XML Schema lets an element declaration carry a `default` or a `fixed` value; an
element so declared that is written empty — `<Monitored/>` — is valid and means the
declared value. `tests/schema_default.rs` reads every such declaration out of the
schemas and checks that the crate reads the declared value out of an empty element
in a real document: it takes every fixture that carries one of those elements,
empties it, reads the document, and checks that the written document carries the
declared value and validates.

Fewer than half of the declared elements occur in the official examples. The
documents below are official examples with the missing elements added, each at the
place and in the order its schema type prescribes, and each carrying an explicit
value so that the document is valid before anything is emptied. The added values are
invented for the occasion; identifiers reuse those of the example they extend.

- **`capability/exd_allServices_capabilitiesRequest_permissions.xml`** — the
  official capabilities request, whose Production Timetable request asks for
  `ParticipantPermissions`.
- **`cm/exc_connectionMonitoring_capabilitiesResponse_policy.xml`** — the official
  Connection Monitoring capabilities, with `ForeignJourneysOnly` in the request
  policy.
- **`cm/exc_connectionMonitoring_request_detail.xml`** — the official Connection
  Monitoring request, its first request naming a `ConnectionMonitoringDetailLevel`.
- **`discovery/exd_stopPoints_discoveryRequest_detail.xml`** — the official stop
  points discovery request, naming a `StopPointsDetailLevel`.
- **`et/ext_estimatedTimetable_response_formation.xml`** — the official Estimated
  Timetable response; its first journey reports a facility change (an
  `EquipmentAvailability` with its `EquipmentStatus`, a `MobilityDisruption` with an
  `AccessFacility`), `EngineOn`, a train formation (`TrainElement` with
  `ReversingDirection` and `SelfPropelled`, `Train` with `TrainSizeType`,
  `TrainComponent` with `ReversedOrientation`), and its first call a `TimingPoint`
  and an `ExpectedDepartureOccupancy` with a `FareClass`.
- **`et/ext_estimatedTimetable_subscriptionRequest_policy.xml`** — the official
  Estimated Timetable subscription request, with every policy element:
  `IncludeInterchanges`, `IncludeJourneyRelations`, `IncludeTrainFormations` and
  `EstimatedTimetableDetailLevel` on the request; `IncrementalUpdates` — `true` by
  default in this service alone — `SkipRecordedCallUpdates` and
  `IncludeOnlyRecordedCallUpdates` on the subscription.
- **`fm/exf_facilityMonitoring_capabilitiesResponse_filters.xml`** — the official
  Facility Monitoring capabilities, with `FilterByVehicleJourneyRef`,
  `FilterByInterchangeRef` and `FilterBySpecificNeed` in the topic filtering,
  `HasMaximumFacilityStatus` in the request policy, and a `ResponseFeatures`
  element with `HasRemedy` and `HasFacilityLocation`.
- **`pt/ext_productionTimetable_capabilitiesResponse_filters.xml`** — the official
  Production Timetable capabilities, with `FilterByVehicleMode` and
  `FilterByProductCategoryRef`.
- **`pt/ext_productionTimetable_response_interchange.xml`** — the official
  Production Timetable response, with a `ServiceJourneyInterchange` between the
  example's journey and an invented return journey, declaring `ExtraInterchange`.
- **`sm/exs_stopMonitoring_capabilitiesResponse_volume.xml`** — the official Stop
  Monitoring capabilities, with `HasMinimumVisitsPerVia` and `HasSituations`.
- **`sm/exs_stopMonitoring_response_rail.xml`** — the official complex Stop
  Monitoring response; the first journey's first via carries a `ViaPriority`, and
  its monitored call the rail properties `ReversesAtStop` and `PlatformTraversal`.
- **`sx/exx_situationExchange_capabilityResponse_filters.xml`** — the official
  Situation Exchange capabilities, with `VisitNumberisOrder`, a
  `TransportDescription` (`CommunicationsTransportMethod`, `CompressionMethod`),
  every topic filter the structure allows (`FilterByMode`, `FilterByNetworkRef`,
  `FilterByVehicleJourneyRef`, `FilterByInterchangeRef`, `FilterBySpecificNeed`,
  `FilterByKeyword`) and `HasMaximumNumberOfSituations`.
- **`sx/exx_situationExchange_request_filters.xml`** — the official Situation
  Exchange request, with `AllowedPredictors` in the request context and, on the
  request, a `ValidityPeriod` with an `EndTimePrecision`,
  `IncludeOnlyIfInPublicationWindow`, and a `RailSubmode` under the example's rail
  `VehicleMode`.
- **`sx/exx_situationExchange_response_scope.xml`** — the official Situation
  Exchange response; its public transport situation gains an `EndTimeStatus` on the
  validity period, `Repetitions` with a `DayType`, eight affected networks — one per
  mode, each carrying its submode element (`AirSubmode`, `BusSubmode`,
  `CoachSubmode`, `MetroSubmode`, `RailSubmode`, `TramSubmode`, `WaterSubmode`,
  `TelecabinSubmode`) — an affected vehicle journey whose route stop points are
  `AffectedOnly` and whose call is a `TimingPoint` with an affected interchange
  (`InterchangeStatusType`) over a connection link (`ConnectionDirection`), and the
  publishing actions to TV (`Ceefax`, `Teletext`), to displays (`OnPlace`,
  `OnBoard`) and by SMS (`Premium`).
- **`vm/exv_vehicleMonitoring_capabilitiesResponse_volume.xml`** — the official
  Vehicle Monitoring capabilities, with `HasMaximumNumberOfCalls` and
  `HasSituations`.

### `ca/` — Control Actions

The standard publishes no Control Actions example at all, so there is nothing to
derive from. The three documents here were written out by the crate's own
Control Actions types — built the way `tests/control_actions.rs` builds its
messages — and are kept as data from then on: they are validated against the
schemas like every other fixture, and the crate has to read them back whole.
Like every Control Actions message, they validate only against `../xsd/siriSg.xsd`,
the substitution-group variant of the root schema; `tests/support` picks that
schema for documents under this directory.

- **`ca/exc_controlAction_capabilitiesResponse.xml`** — a capabilities response with
  `FilterByMode`, `FilterByNetworkRef`, `HasMaximumControlActions` and
  `HasSituations`. `HasDriverMessages` and `HasVehicleDetectings` are absent: the
  schema declares them in the request policy, the crate writes them in the
  response features, and a document carrying them there would not validate;
  `tests/schema_default.rs` records the two as not exercisable.
- **`ca/exc_controlAction_request.xml`** — a service request for control actions
  naming `IncludeDriverMessages` and `IncludeVehicleDetectings`.
- **`ca/exc_controlAction_delivery.xml`** — a delivery of four control actions — a
  partial journey cancellation (`ConcernsArrivals`, `ConcernsDepartures`), a change
  of journey timing whose relative time names a `ChangeModel`, an extra connection
  (`IsExposedToStaff`, `IsExposedToPassengers`) and a vehicle work assignment
  (`DeAssignment`) — and a driver message (`OriginatedByDriver`, `CallForMeans`,
  `CallForRepairs`).
