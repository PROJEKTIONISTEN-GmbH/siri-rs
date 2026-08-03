# Conformance fixtures

Everything in this directory is a verbatim copy of published CEN SIRI material.
Nothing here has been edited; the point of the harness is that the crate has to
match these documents, not the other way round.

- **Source**: the SIRI XML Schema repository, <https://github.com/SIRI-CEN/SIRI>
- **Version**: v2.2
- **Copyright**: © 2006–2026 NeTEx, CEN, Crown Copyright

## `xsd/`

The complete schema set, copied unchanged. The root is `xsd/siri.xsd`; the
validation tests point `xmllint` at it. The `wsdl_model/` directory and the `.wsdl`
files are omitted because they describe SOAP bindings rather than the document
grammar, and nothing here validates against them.

## `xml/`

The official example documents, grouped by the part of the standard they exercise.

| Directory | Source directory in the SIRI repository | Documents |
|---|---|---|
| `framework/` | `examples/siri_exa_framework` | 12 |
| `pt/` | `examples/siri_exm_PT` | 4 |
| `et/` | `examples/siri_exm_ET` | 4 |
| `vm/` | `examples/siri_exm_VM` | 7 |
| `sm/` | `examples/siri_exm_SM` | 9 |
| `st/` | `examples/siri_exm_ST` | 4 |
| `cm/` | `examples/siri_exm_CM` | 5 |
| `ct/` | `examples/siri_exm_CT` | 4 |
| `gm/` | `examples/siri_exm_GM` | 5 |
| `fm/` | `examples/siri_exm_FM` | 4 |
| `sx/` | `examples/siri_exm_SX` | 9 |
| `sx/vdv736/` | `examples/siri_exm_SX/VDV736_exm` | 4 |
| `discovery/` | `examples/siri_exu_discovery` | 10 |
| `capability/` | `examples/siri_exu_capability` | 1 |

`sx/vdv736/` is the German SIRI-SX profile, VDV 736: four messages tracing one
incident from first report through two updates to its closure.

## Not included

`examples/siri_exu_capability/exd_allServices_capabilitiesResponse.xml` is the only
document in the directories above that is not here. It states the capabilities of
all eleven functional services in one message, and one of them — Situation Exchange
Discovery — would need a capability structure of its own before the document could
be read. The companion request document, `exd_allServices_capabilitiesRequest.xml`,
*is* included — a capability request carries no service-specific content, so all
eleven are supported.

`examples/occupancy` is out of scope: those documents exercise the occupancy
extension profile rather than a functional service.

The Control Actions service ships no example messages at all, so it is covered by
the schemas alone; `src/ca` says so in its own documentation.

No VDV 454 example messages are shipped with the schema repository, so the German
profile of the timetable services is covered here only in so far as the schemas
carry it: the elements VDV asked for are modelled and validate, but there is no
profile message to round-trip the way `sx/vdv736/` does for Situation Exchange.
