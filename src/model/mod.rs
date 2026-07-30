//! Model elements shared by every SIRI service: references, positions, features.

mod accessibility;
mod feature;
mod journey;
mod location;
mod mode;
mod projection;
mod reference;

pub use accessibility::{
    AccessibilityAssessment, AccessibilityLimitation, AccessibilityLimitations,
    PassengerAccessibilityNeeds, Suitabilities, Suitability, Timeband, Timebands, UserNeed,
    UserNeedKind, ValidityCondition,
};
pub use feature::{ProductCategory, ServiceFeature, VehicleFeature};
pub use journey::{
    CompoundTrainRef, Direction, JourneyPartInfo, JourneyPartRef, TrainBlockPart, TrainNumberRef,
    TrainPartRef,
};
pub use location::{BoundingBox, LineShape, Location, Position};
pub use mode::Submode;
pub use projection::{
    GisFeatureRef, GisFeatures, LinkProjection, PointProjection, ProjectionBoundary, ProjectionLine,
    ZoneProjection,
};
pub use reference::{
    ConnectionLinkRef, ControlActionRef, DataFrameRef, DatedVehicleJourneyRef, DestinationRef,
    DirectionRef, FacilityRef, FramedVehicleJourneyRef, InterchangeRef, JourneyPatternRef,
    LineDirection, LineRef, OperationalUnitRef, OperatorRef, PlaceRef, ProductCategoryRef, RouteRef,
    ServiceFeatureRef, StopAreaRef, StopPlaceComponentRef, StopPlaceRef, StopPointRef,
    VehicleFeatureRef, VehicleJourneyRef, VehicleRef,
};
