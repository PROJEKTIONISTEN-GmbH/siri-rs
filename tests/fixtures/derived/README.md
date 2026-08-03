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
