//! Geographic positions.

use serde::{Deserialize, Serialize};

/// A point on the earth.
///
/// The schema offers two ways to state a position — WGS 84 longitude/latitude, or
/// a `Coordinates` string in the projection named by `srs_name` — so the fields
/// for both are optional. Build one with [`Location::wgs84`] or
/// [`Location::coordinates`] and read back which form is present with
/// [`Location::position`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Location {
    /// Identifier of this location within the document.
    #[serde(rename = "@id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Name of the spatial reference system `coordinates` is expressed in.
    #[serde(rename = "@srsName", default, skip_serializing_if = "Option::is_none")]
    pub srs_name: Option<String>,
    /// Degrees east of the prime meridian, between -180 and 180.
    #[serde(rename = "Longitude", default, skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    /// Degrees north of the equator, between -90 and 90.
    #[serde(rename = "Latitude", default, skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    /// Metres above sea level, between -1000 and 5000.
    #[serde(rename = "Altitude", default, skip_serializing_if = "Option::is_none")]
    pub altitude: Option<f64>,
    /// The position in the projection named by `srs_name`, as a coordinate list.
    #[serde(rename = "Coordinates", default, skip_serializing_if = "Option::is_none")]
    pub coordinates: Option<String>,
    /// Precision of the position in metres.
    #[serde(rename = "Precision", default, skip_serializing_if = "Option::is_none")]
    pub precision: Option<u64>,
}

/// Which of the two ways of stating a position a [`Location`] uses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position<'a> {
    /// WGS 84 degrees, with an optional altitude in metres.
    Wgs84 {
        /// Degrees east of the prime meridian.
        longitude: f64,
        /// Degrees north of the equator.
        latitude: f64,
        /// Metres above sea level.
        altitude: Option<f64>,
    },
    /// A coordinate list in the projection named by the location's `srs_name`.
    Coordinates(&'a str),
}

impl Location {
    /// A WGS 84 position.
    pub fn wgs84(longitude: f64, latitude: f64) -> Self {
        Self {
            longitude: Some(longitude),
            latitude: Some(latitude),
            ..Self::default()
        }
    }

    /// A position in the projection named by `srs_name`.
    pub fn coordinates(srs_name: impl Into<String>, coordinates: impl Into<String>) -> Self {
        Self {
            srs_name: Some(srs_name.into()),
            coordinates: Some(coordinates.into()),
            ..Self::default()
        }
    }

    /// Which alternative of the schema's choice this location carries, or `None`
    /// when neither is present.
    pub fn position(&self) -> Option<Position<'_>> {
        match (self.longitude, self.latitude, self.coordinates.as_deref()) {
            (Some(longitude), Some(latitude), _) => Some(Position::Wgs84 {
                longitude,
                latitude,
                altitude: self.altitude,
            }),
            (_, _, Some(coordinates)) => Some(Position::Coordinates(coordinates)),
            _ => None,
        }
    }
}

/// A rectangular area, given by two opposite corners.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    /// North-west corner.
    #[serde(rename = "UpperLeft")]
    pub upper_left: Location,
    /// South-east corner.
    #[serde(rename = "LowerRight")]
    pub lower_right: Location,
}

impl BoundingBox {
    /// A box spanning the two given corners.
    pub fn new(upper_left: Location, lower_right: Location) -> Self {
        Self {
            upper_left,
            lower_right,
        }
    }
}

/// An open line, given as an ordered list of at least two points.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LineShape {
    /// The points along the line, in order of travel.
    #[serde(rename = "Point", default, skip_serializing_if = "Vec::is_empty")]
    pub point: Vec<Location>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wgs84_location_reports_its_position() {
        let location = Location::wgs84(0.1, 53.0);
        assert_eq!(
            location.position(),
            Some(Position::Wgs84 {
                longitude: 0.1,
                latitude: 53.0,
                altitude: None
            })
        );
    }

    #[test]
    fn a_projected_location_reports_its_coordinate_list() {
        let location = Location::coordinates("EPSG:27700", "530000 180000");
        assert_eq!(
            location.position(),
            Some(Position::Coordinates("530000 180000"))
        );
    }

    #[test]
    fn an_empty_location_reports_no_position() {
        assert_eq!(Location::default().position(), None);
    }
}
