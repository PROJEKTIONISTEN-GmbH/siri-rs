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
#[non_exhaustive]
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

/// A circle, given by its centre and how far it reaches.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CircularArea {
    /// Identifier of this area within the document.
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
    /// The centre in the projection named by `srs_name`, as a coordinate list.
    #[serde(rename = "Coordinates", default, skip_serializing_if = "Option::is_none")]
    pub coordinates: Option<String>,
    /// Precision of the centre in metres.
    #[serde(rename = "Precision", default, skip_serializing_if = "Option::is_none")]
    pub precision: Option<u64>,
    /// How far the area reaches from the centre, in metres.
    #[serde(rename = "Radius", default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<u64>,
}

/// An area a service may be hailed or booked within, rather than a fixed stop.
///
/// The schema offers three ways to draw one and the fields for all three are
/// optional; [`FlexibleArea::shape`] reports which is present.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FlexibleArea {
    /// The area as a rectangle.
    #[serde(rename = "BoundingBox", default, skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<BoundingBox>,
    /// The area as a circle.
    #[serde(rename = "CircularArea", default, skip_serializing_if = "Option::is_none")]
    pub circular_area: Option<CircularArea>,
    /// The area as a GML polygon.
    #[serde(rename = "gml:Polygon", alias = "Polygon", default, skip_serializing_if = "Option::is_none")]
    pub polygon: Option<Polygon>,
}

/// Which of the three ways of drawing a [`FlexibleArea`] is used.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum FlexibleShape<'a> {
    /// A rectangle spanned by two opposite corners.
    BoundingBox(&'a BoundingBox),
    /// A circle around a centre.
    CircularArea(&'a CircularArea),
    /// A GML polygon.
    Polygon(&'a Polygon),
}

impl FlexibleArea {
    /// A rectangular area.
    pub fn bounding_box(bounding_box: BoundingBox) -> Self {
        Self {
            bounding_box: Some(bounding_box),
            ..Self::default()
        }
    }

    /// A circular area.
    pub fn circular_area(circular_area: CircularArea) -> Self {
        Self {
            circular_area: Some(circular_area),
            ..Self::default()
        }
    }

    /// An area drawn as a polygon.
    pub fn polygon(polygon: Polygon) -> Self {
        Self {
            polygon: Some(polygon),
            ..Self::default()
        }
    }

    /// Which alternative of the schema's choice this area carries, or `None` when
    /// none of them is present.
    pub fn shape(&self) -> Option<FlexibleShape<'_>> {
        self.bounding_box
            .as_ref()
            .map(FlexibleShape::BoundingBox)
            .or_else(|| self.circular_area.as_ref().map(FlexibleShape::CircularArea))
            .or_else(|| self.polygon.as_ref().map(FlexibleShape::Polygon))
    }
}

/// The GML 3.2 namespace, `http://www.opengis.net/gml/3.2`.
pub const GML_NAMESPACE: &str = "http://www.opengis.net/gml/3.2";

fn gml_namespace() -> String {
    GML_NAMESPACE.to_owned()
}

/// A polygon, the one geometry SIRI borrows from GML rather than modelling itself.
///
/// A polygon is one outer ring and any number of holes. Its elements are in the GML
/// namespace rather than SIRI's, so each polygon carries the namespace binding its
/// own subtree needs — a SIRI document that declares GML on an ancestor is read
/// just as well, and written back with the declaration repeated here, which XML
/// treats as the same document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polygon {
    /// Binding of the `gml` prefix used by this element and everything under it.
    #[serde(rename = "@xmlns:gml", default = "gml_namespace")]
    pub gml_namespace: String,
    /// Identifier of this geometry within the document; GML requires one.
    #[serde(rename = "@gml:id", alias = "@id")]
    pub id: String,
    /// Name of the spatial reference system the positions are expressed in.
    #[serde(rename = "@srsName", default, skip_serializing_if = "Option::is_none")]
    pub srs_name: Option<String>,
    /// The outer boundary.
    #[serde(rename = "gml:exterior", alias = "exterior", default, skip_serializing_if = "Option::is_none")]
    pub exterior: Option<RingProperty>,
    /// The boundaries of the holes cut out of it.
    #[serde(rename = "gml:interior", alias = "interior", default, skip_serializing_if = "Vec::is_empty")]
    pub interior: Vec<RingProperty>,
}

impl Polygon {
    /// A polygon with the given outer boundary and no holes.
    pub fn new(id: impl Into<String>, exterior: LinearRing) -> Self {
        Self {
            gml_namespace: gml_namespace(),
            id: id.into(),
            srs_name: None,
            exterior: Some(RingProperty {
                linear_ring: exterior,
            }),
            interior: Vec::new(),
        }
    }
}

/// One boundary of a [`Polygon`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RingProperty {
    /// The ring itself.
    #[serde(rename = "gml:LinearRing", alias = "LinearRing")]
    pub linear_ring: LinearRing,
}

/// A closed ring of positions.
///
/// The positions are the coordinate list GML calls `posList`: pairs of numbers,
/// separated by spaces, the last pair repeating the first to close the ring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearRing {
    /// The positions along the ring.
    #[serde(rename = "gml:posList", alias = "posList")]
    pub pos_list: PosList,
}

impl LinearRing {
    /// A ring through the given positions.
    pub fn new(positions: impl Into<String>) -> Self {
        Self {
            pos_list: PosList {
                srs_dimension: None,
                count: None,
                value: positions.into(),
            },
        }
    }
}

/// A list of positions, written as one whitespace-separated run of numbers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PosList {
    /// How many numbers make up one position; two unless stated otherwise.
    #[serde(rename = "@srsDimension", default, skip_serializing_if = "Option::is_none")]
    pub srs_dimension: Option<u64>,
    /// How many positions the list holds.
    #[serde(rename = "@count", default, skip_serializing_if = "Option::is_none")]
    pub count: Option<u64>,
    /// The numbers themselves.
    #[serde(rename = "$text")]
    pub value: String,
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

    #[test]
    fn a_polygon_carries_the_gml_namespace_it_needs() {
        let area = FlexibleArea::polygon(Polygon::new(
            "area-1",
            LinearRing::new("53.55 0.1 53.56 0.1 53.56 0.2 53.55 0.1"),
        ));

        let xml = quick_xml::se::to_string_with_root("AimedFlexibleArea", &area).unwrap();
        assert!(
            xml.contains(r#"<gml:Polygon xmlns:gml="http://www.opengis.net/gml/3.2" gml:id="area-1">"#),
            "{xml}"
        );
        assert!(
            xml.contains("<gml:posList>53.55 0.1 53.56 0.1 53.56 0.2 53.55 0.1</gml:posList>"),
            "{xml}"
        );

        let read: FlexibleArea = quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(read, area);
        assert!(matches!(read.shape(), Some(FlexibleShape::Polygon(_))));
    }

    #[test]
    fn a_polygon_that_inherited_its_namespace_is_read_and_written_back_with_one() {
        let inherited = r#"<AimedFlexibleArea><gml:Polygon gml:id="area-2">
            <gml:exterior><gml:LinearRing><gml:posList srsDimension="2" count="4">0 0 1 0 1 1 0 0</gml:posList></gml:LinearRing></gml:exterior>
        </gml:Polygon></AimedFlexibleArea>"#;

        let area: FlexibleArea = quick_xml::de::from_str(inherited).unwrap();
        let polygon = area.polygon.as_ref().unwrap();
        assert_eq!(polygon.gml_namespace, GML_NAMESPACE);
        assert_eq!(polygon.exterior.as_ref().unwrap().linear_ring.pos_list.count, Some(4));

        let written = quick_xml::se::to_string_with_root("AimedFlexibleArea", &area).unwrap();
        assert!(written.contains(r#"xmlns:gml="http://www.opengis.net/gml/3.2""#), "{written}");
    }

    #[test]
    fn a_flexible_area_reports_which_shape_it_was_drawn_with() {
        let circle = FlexibleArea::circular_area(CircularArea {
            radius: Some(300),
            ..CircularArea::default()
        });
        assert!(matches!(circle.shape(), Some(FlexibleShape::CircularArea(_))));

        let box_ = FlexibleArea::bounding_box(BoundingBox::new(
            Location::wgs84(0.0, 53.0),
            Location::wgs84(0.1, 52.9),
        ));
        assert!(matches!(box_.shape(), Some(FlexibleShape::BoundingBox(_))));
        assert_eq!(FlexibleArea::default().shape(), None);
    }
}
