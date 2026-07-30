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
| `sx/` | `examples/siri_exm_SX` | 9 |
| `sx/vdv736/` | `examples/siri_exm_SX/VDV736_exm` | 4 |
| `discovery/` | `examples/siri_exu_discovery` | 10 |
| `capability/` | `examples/siri_exu_capability` | 1 |

`sx/vdv736/` is the German SIRI-SX profile, VDV 736: four messages tracing one
incident from first report through two updates to its closure.

## Not included

`examples/siri_exu_capability/exd_allServices_capabilitiesResponse.xml` is the only
document in the directories above that is not here. It states the capabilities of
all eleven functional services, ten of which this release does not implement;
including it would mean modelling ten capability structures for services whose
messages the crate cannot read anyway. The companion request document,
`exd_allServices_capabilitiesRequest.xml`, *is* included — a capability request
carries no service-specific content, so all eleven are supported.

The example directories for the other functional services
(`examples/siri_exm_{PT,ET,ST,SM,VM,CT,CM,GM,FM}` and `examples/occupancy`) are out
of scope for the same reason and will arrive with those services.
