//! Projections: how a transport element is drawn onto a map.
//!
//! A projection ties a network element — a link between two stops, a point, a fare
//! zone — either to features of the producer's geographic information system or to
//! an explicit list of coordinates. Consumers that render maps use whichever form
//! they can resolve.

use serde::{Deserialize, Serialize};

/// A path between two points.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LinkProjection {
    /// Features of the geographic information system the path runs along.
    #[serde(rename = "Features", default, skip_serializing_if = "Option::is_none")]
    pub features: Option<GisFeatures>,
    /// The path as an explicit sequence of points.
    #[serde(rename = "Line", default, skip_serializing_if = "Option::is_none")]
    pub line: Option<ProjectionLine>,
}

/// An area, given as a closed boundary.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ZoneProjection {
    /// Features of the geographic information system the area covers.
    #[serde(rename = "Features", default, skip_serializing_if = "Option::is_none")]
    pub features: Option<GisFeatures>,
    /// The boundary; the schema requires at least three points per ring.
    #[serde(rename = "Boundary", default, skip_serializing_if = "Vec::is_empty")]
    pub boundary: Vec<ProjectionBoundary>,
}

/// One closed ring of a [`ZoneProjection`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectionBoundary {
    /// The points of the ring, in order.
    #[serde(rename = "PointProjection")]
    pub point_projection: Vec<PointProjection>,
}

/// The features of a geographic information system an element projects onto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GisFeatures {
    /// The features, at least one.
    #[serde(rename = "GisFeatureRef")]
    pub gis_feature_ref: Vec<GisFeatureRef>,
}

/// A reference to one feature of a geographic information system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GisFeatureRef {
    /// The system's own identifier for the feature.
    #[serde(rename = "FeatureIdRef")]
    pub feature_id_ref: String,
    /// What kind of feature it is, in the system's own vocabulary.
    #[serde(rename = "FeatureType", default, skip_serializing_if = "Option::is_none")]
    pub feature_type: Option<String>,
}

/// A projected path, given as an ordered sequence of points.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectionLine {
    /// The points; the schema requires at least two, a start and an end.
    #[serde(rename = "PointProjection")]
    pub point_projection: Vec<PointProjection>,
}

/// One point of a projection.
///
/// As with a [`Location`](crate::model::Location), the position is given either as
/// WGS 84 degrees or as a coordinate list in the projection named by `srs_name`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PointProjection {
    /// Identifier of this point within the document.
    #[serde(rename = "@id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Name of the spatial reference system `coordinates` is expressed in.
    #[serde(rename = "@srsName", default, skip_serializing_if = "Option::is_none")]
    pub srs_name: Option<String>,
    /// Features of the geographic information system this point projects onto.
    #[serde(rename = "Features", default, skip_serializing_if = "Option::is_none")]
    pub features: Option<GisFeatures>,
    /// Degrees east of the prime meridian.
    #[serde(rename = "Longitude", default, skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    /// Degrees north of the equator.
    #[serde(rename = "Latitude", default, skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    /// Metres above sea level.
    #[serde(rename = "Altitude", default, skip_serializing_if = "Option::is_none")]
    pub altitude: Option<f64>,
    /// The position in the projection named by `srs_name`, as a coordinate list.
    #[serde(rename = "Coordinates", default, skip_serializing_if = "Option::is_none")]
    pub coordinates: Option<String>,
    /// Precision of the position in metres.
    #[serde(rename = "Precision", default, skip_serializing_if = "Option::is_none")]
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
