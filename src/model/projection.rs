//! Projections: how a transport element is drawn onto a map.
//!
//! A projection ties a network element — a link between two stops, a point, a fare
//! zone — either to features of the producer's geographic information system or to
//! an explicit list of coordinates. Consumers that render maps use whichever form
//! they can resolve.
//!
//! # Namespace
//!
//! The projection types are IFOPT's, and so is everything inside them: SIRI names
//! the element that carries a link projection — `LinkProjectionToNextStopPoint`,
//! `LinkProjection` in an affected section — in its own namespace, but `Features`,
//! `Line`, `Boundary` and each `PointProjection` below it are `ifopt:` elements. A
//! stop place component goes further and carries the IFOPT elements themselves
//! (`ifopt:PointProjection`, `ifopt:LinkProjection`, `ifopt:ZoneProjection`).
//!
//! A [`LinkProjection`], [`ZoneProjection`] or [`PointProjection`] is therefore
//! written with its content under the `ifopt` prefix, and binds that prefix on the
//! element it is written as, so that it is right wherever it is carried. The
//! points of a line or a boundary use the binding of the projection they belong to.
//!
//! Releases up to 2.4 wrote all of this in the SIRI namespace, which the schema
//! rejects. A document written that way still reads: the reader goes by an
//! element's local name, so it reads both forms alike.

use serde::{Deserialize, Serialize, Serializer};

use crate::model::accessibility::IFOPT_NAMESPACE;

/// A path between two points.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct LinkProjection {
    /// Features of the geographic information system the path runs along.
    #[serde(rename = "Features", default)]
    pub features: Option<GisFeatures>,
    /// The path as an explicit sequence of points.
    #[serde(rename = "Line", default)]
    pub line: Option<ProjectionLine>,
}

impl Serialize for LinkProjection {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Written<'a> {
            #[serde(rename = "@xmlns:ifopt")]
            ifopt_namespace: &'static str,
            #[serde(rename = "ifopt:Features", skip_serializing_if = "Option::is_none")]
            features: Option<&'a GisFeatures>,
            #[serde(rename = "ifopt:Line", skip_serializing_if = "Option::is_none")]
            line: Option<&'a ProjectionLine>,
        }

        Written {
            ifopt_namespace: IFOPT_NAMESPACE,
            features: self.features.as_ref(),
            line: self.line.as_ref(),
        }
        .serialize(serializer)
    }
}

/// An area, given as a closed boundary.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ZoneProjection {
    /// Features of the geographic information system the area covers.
    #[serde(rename = "Features", default)]
    pub features: Option<GisFeatures>,
    /// The boundary; the schema requires at least three points per ring.
    #[serde(rename = "Boundary", default)]
    pub boundary: Vec<ProjectionBoundary>,
}

impl Serialize for ZoneProjection {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Written<'a> {
            #[serde(rename = "@xmlns:ifopt")]
            ifopt_namespace: &'static str,
            #[serde(rename = "ifopt:Features", skip_serializing_if = "Option::is_none")]
            features: Option<&'a GisFeatures>,
            #[serde(rename = "ifopt:Boundary", skip_serializing_if = "<[_]>::is_empty")]
            boundary: &'a [ProjectionBoundary],
        }

        Written {
            ifopt_namespace: IFOPT_NAMESPACE,
            features: self.features.as_ref(),
            boundary: &self.boundary,
        }
        .serialize(serializer)
    }
}

/// One closed ring of a [`ZoneProjection`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectionBoundary {
    /// The points of the ring, in order.
    #[serde(
        rename = "ifopt:PointProjection",
        alias = "PointProjection",
        serialize_with = "points_of_a_projection"
    )]
    pub point_projection: Vec<PointProjection>,
}

/// The features of a geographic information system an element projects onto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GisFeatures {
    /// The features, at least one.
    #[serde(rename = "ifopt:GisFeatureRef", alias = "GisFeatureRef")]
    pub gis_feature_ref: Vec<GisFeatureRef>,
}

/// A reference to one feature of a geographic information system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GisFeatureRef {
    /// The system's own identifier for the feature.
    #[serde(rename = "ifopt:FeatureIdRef", alias = "FeatureIdRef")]
    pub feature_id_ref: String,
    /// What kind of feature it is, in the system's own vocabulary.
    #[serde(
        rename = "ifopt:FeatureType",
        alias = "FeatureType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub feature_type: Option<String>,
}

/// A projected path, given as an ordered sequence of points.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectionLine {
    /// The points; the schema requires at least two, a start and an end.
    #[serde(
        rename = "ifopt:PointProjection",
        alias = "PointProjection",
        serialize_with = "points_of_a_projection"
    )]
    pub point_projection: Vec<PointProjection>,
}

/// One point of a projection.
///
/// As with a [`Location`](crate::model::Location), the position is given either as
/// WGS 84 degrees or as a coordinate list in the projection named by `srs_name`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct PointProjection {
    /// Identifier of this point within the document.
    #[serde(rename = "@id", default)]
    pub id: Option<String>,
    /// Name of the spatial reference system `coordinates` is expressed in.
    #[serde(rename = "@srsName", default)]
    pub srs_name: Option<String>,
    /// Features of the geographic information system this point projects onto.
    #[serde(rename = "Features", default)]
    pub features: Option<GisFeatures>,
    /// Degrees east of the prime meridian.
    #[serde(rename = "Longitude", default)]
    pub longitude: Option<f64>,
    /// Degrees north of the equator.
    #[serde(rename = "Latitude", default)]
    pub latitude: Option<f64>,
    /// Metres above sea level.
    #[serde(rename = "Altitude", default)]
    pub altitude: Option<f64>,
    /// The position in the projection named by `srs_name`, as a coordinate list.
    #[serde(rename = "Coordinates", default)]
    pub coordinates: Option<String>,
    /// Precision of the position in metres.
    #[serde(rename = "Precision", default)]
    pub precision: Option<u64>,
}

impl PointProjection {
    /// A point at the given WGS 84 position.
    pub fn wgs84(longitude: f64, latitude: f64) -> Self {
        Self {
            longitude: Some(longitude),
            latitude: Some(latitude),
            ..Self::default()
        }
    }
}

impl Serialize for PointProjection {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        WrittenPoint::new(self, Some(IFOPT_NAMESPACE)).serialize(serializer)
    }
}

/// A [`PointProjection`] as it is written: its content under the `ifopt` prefix,
/// and the prefix bound on the point itself unless the projection it belongs to
/// has bound it already.
#[derive(Serialize)]
struct WrittenPoint<'a> {
    #[serde(rename = "@xmlns:ifopt", skip_serializing_if = "Option::is_none")]
    ifopt_namespace: Option<&'static str>,
    #[serde(rename = "@id", skip_serializing_if = "Option::is_none")]
    id: Option<&'a str>,
    #[serde(rename = "@srsName", skip_serializing_if = "Option::is_none")]
    srs_name: Option<&'a str>,
    #[serde(rename = "ifopt:Features", skip_serializing_if = "Option::is_none")]
    features: Option<&'a GisFeatures>,
    #[serde(rename = "ifopt:Longitude", skip_serializing_if = "Option::is_none")]
    longitude: Option<f64>,
    #[serde(rename = "ifopt:Latitude", skip_serializing_if = "Option::is_none")]
    latitude: Option<f64>,
    #[serde(rename = "ifopt:Altitude", skip_serializing_if = "Option::is_none")]
    altitude: Option<f64>,
    #[serde(rename = "ifopt:Coordinates", skip_serializing_if = "Option::is_none")]
    coordinates: Option<&'a str>,
    #[serde(rename = "ifopt:Precision", skip_serializing_if = "Option::is_none")]
    precision: Option<u64>,
}

impl<'a> WrittenPoint<'a> {
    fn new(point: &'a PointProjection, ifopt_namespace: Option<&'static str>) -> Self {
        Self {
            ifopt_namespace,
            id: point.id.as_deref(),
            srs_name: point.srs_name.as_deref(),
            features: point.features.as_ref(),
            longitude: point.longitude,
            latitude: point.latitude,
            altitude: point.altitude,
            coordinates: point.coordinates.as_deref(),
            precision: point.precision,
        }
    }
}

/// Writes the points of a line or a boundary, which the projection they belong to
/// has already bound the `ifopt` prefix for.
fn points_of_a_projection<S: Serializer>(points: &[PointProjection], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_seq(points.iter().map(|point| WrittenPoint::new(point, None)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DECLARATION: &str = r#"xmlns:ifopt="http://www.ifopt.org.uk/ifopt""#;

    fn two_point_link() -> LinkProjection {
        LinkProjection {
            features: None,
            line: Some(ProjectionLine {
                point_projection: vec![PointProjection::wgs84(9.7, 52.4), PointProjection::wgs84(9.8, 52.5)],
            }),
        }
    }

    #[test]
    fn a_link_projection_binds_the_ifopt_prefix_once_for_everything_under_it() {
        let xml = quick_xml::se::to_string_with_root("LinkProjectionToNextStopPoint", &two_point_link()).unwrap();

        assert!(
            xml.starts_with(&format!(
                "<LinkProjectionToNextStopPoint {DECLARATION}><ifopt:Line><ifopt:PointProjection>\
                 <ifopt:Longitude>9.7</ifopt:Longitude><ifopt:Latitude>52.4</ifopt:Latitude>"
            )),
            "{xml}"
        );
        assert_eq!(xml.matches("xmlns:ifopt").count(), 1, "{xml}");
    }

    #[test]
    fn a_zone_projection_writes_its_features_and_rings_under_the_prefix() {
        let zone = ZoneProjection {
            features: Some(GisFeatures {
                gis_feature_ref: vec![GisFeatureRef {
                    feature_id_ref: "PLATFORM-3".to_owned(),
                    feature_type: Some("area".to_owned()),
                }],
            }),
            boundary: vec![ProjectionBoundary {
                point_projection: vec![
                    PointProjection::wgs84(9.70, 52.40),
                    PointProjection::wgs84(9.71, 52.40),
                    PointProjection::wgs84(9.71, 52.41),
                ],
            }],
        };
        let xml = quick_xml::se::to_string_with_root("ifopt:ZoneProjection", &zone).unwrap();

        assert!(
            xml.starts_with(&format!(
                "<ifopt:ZoneProjection {DECLARATION}><ifopt:Features><ifopt:GisFeatureRef>\
                 <ifopt:FeatureIdRef>PLATFORM-3</ifopt:FeatureIdRef><ifopt:FeatureType>area</ifopt:FeatureType>\
                 </ifopt:GisFeatureRef></ifopt:Features><ifopt:Boundary><ifopt:PointProjection>"
            )),
            "{xml}"
        );
        assert_eq!(xml.matches("xmlns:ifopt").count(), 1, "{xml}");
    }

    #[test]
    fn a_point_projection_of_its_own_binds_the_prefix_itself() {
        let point = PointProjection {
            id: Some("P1".to_owned()),
            srs_name: Some("EPSG:25832".to_owned()),
            coordinates: Some("550123.4 5874321.2".to_owned()),
            precision: Some(2),
            ..PointProjection::default()
        };
        let xml = quick_xml::se::to_string_with_root("ifopt:PointProjection", &point).unwrap();

        assert_eq!(
            xml,
            format!(
                "<ifopt:PointProjection {DECLARATION} id=\"P1\" srsName=\"EPSG:25832\">\
                 <ifopt:Coordinates>550123.4 5874321.2</ifopt:Coordinates>\
                 <ifopt:Precision>2</ifopt:Precision></ifopt:PointProjection>"
            )
        );
    }

    #[test]
    fn a_projection_reads_alike_with_and_without_the_prefix() {
        let prefixed = format!(
            "<LinkProjection {DECLARATION}><ifopt:Features><ifopt:GisFeatureRef>\
             <ifopt:FeatureIdRef>RL-1</ifopt:FeatureIdRef></ifopt:GisFeatureRef></ifopt:Features>\
             <ifopt:Line><ifopt:PointProjection><ifopt:Longitude>9.7</ifopt:Longitude>\
             <ifopt:Latitude>52.4</ifopt:Latitude></ifopt:PointProjection><ifopt:PointProjection>\
             <ifopt:Longitude>9.8</ifopt:Longitude><ifopt:Latitude>52.5</ifopt:Latitude>\
             </ifopt:PointProjection></ifopt:Line></LinkProjection>"
        );
        let unprefixed = prefixed
            .replace(&format!(" {DECLARATION}"), "")
            .replace("<ifopt:", "<")
            .replace("</ifopt:", "</");

        let from_prefixed: LinkProjection = quick_xml::de::from_str(&prefixed).unwrap();
        let from_unprefixed: LinkProjection = quick_xml::de::from_str(&unprefixed).unwrap();
        let expected = LinkProjection {
            features: Some(GisFeatures {
                gis_feature_ref: vec![GisFeatureRef {
                    feature_id_ref: "RL-1".to_owned(),
                    feature_type: None,
                }],
            }),
            ..two_point_link()
        };
        assert_eq!(from_prefixed, expected);
        assert_eq!(from_unprefixed, expected);
    }
}
