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
