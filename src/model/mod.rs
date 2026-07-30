//! Model elements shared by every SIRI service: references, positions, journeys.

mod accessibility;
mod call;
mod dated_journey;
mod estimated_journey;
mod feature;
mod formation;
mod journey;
mod location;
mod mode;
mod monitored_journey;
mod projection;
mod reference;

pub use accessibility::{
    AccessibilityAssessment, AccessibilityLimitation, AccessibilityLimitations,
    PassengerAccessibilityNeeds, Suitabilities, Suitability, Timeband, Timebands, UserNeed,
    UserNeedKind, ValidityCondition,
};
pub use call::{
    BoardingPositionRef, FlexibleAreaRef, PlannedStopAssignment, StandingPlace, StopAssignment,
};
pub use dated_journey::{
    ContextualisedConnectionLink, DatedCall, DatedCalls, DatedVehicleJourney,
    FromServiceJourneyInterchange, RemovedDatedVehicleJourney, RemovedServiceJourneyInterchange,
    ServiceJourneyInterchange, TargetedInterchange, ToServiceJourneyInterchange,
};
pub use estimated_journey::{
    CallAlteration, EstimatedCall, EstimatedCalls, EstimatedServiceJourneyInterchange,
    EstimatedVehicleJourney, JourneyAlteration, JourneyIdentity, RecordedCall, RecordedCalls,
    WillWait,
};
pub use feature::{ProductCategory, ServiceFeature, VehicleFeature};
pub use formation::{
    CompoundTrain, CompoundTrainItem, CompoundTrains, EntranceToVehicleRef, FormationAssignment, FormationCondition,
    FormationConditionStatus, FormationStatus, GroupReservation, MaximumPassengerCapacities,
    PassageBetweenTrains, Passages, PassengerCapacity, RecommendedAction, Train, TrainComponent,
    TrainComponentItem, TrainComponentRef, TrainComponents, TrainElement, TrainElementItem,
    TrainElementRef, TrainElements, TrainFormationReference, TrainInCompoundTrain,
    TrainInCompoundTrainItem, TrainInCompoundTrainRef, TrainItem, TrainRef, Trains,
    TrainsInCompoundTrain, TypeOfActionRef, VehicleInFormationStatusRecord, VehicleOccupancy,
};
pub use journey::{
    Branding, BrandingRef, CompoundTrainRef, ConnectingJourneyRef, DatedVehicleJourneyIndirectRef,
    Direction, GroupOfLinesRef, JourneyPartInfo, JourneyPartRef, JourneyParts, JourneyPlaceRef,
    JourneyRelation, JourneyRelationScope, JourneyRelations, PredictionQuality,
    ProgressBetweenStops, RelatedCall, RelatedJourney, SimpleContact, TrainBlockPart,
    TrainNumberRef, TrainNumbers, TrainPartRef, ViaName,
};
pub use location::{
    BoundingBox, CircularArea, FlexibleArea, FlexibleShape, LineShape, LinearRing, Location,
    PosList, Polygon, Position, RingProperty, GML_NAMESPACE,
};
pub use mode::Submode;
pub use monitored_journey::{
    MonitoredCall, MonitoredVehicleJourney, OnwardCall, OnwardCalls, PreviousCall, PreviousCalls,
};
pub use projection::{
    GisFeatureRef, GisFeatures, LinkProjection, PointProjection, ProjectionBoundary, ProjectionLine,
    ZoneProjection,
};
pub use reference::{
    BlockRef, ConnectionLinkRef, ControlActionRef, CourseOfJourneyRef, DataFrameRef,
    DatedVehicleJourneyRef, DestinationRef, DirectionRef, FacilityRef, FramedVehicleJourneyRef,
    InterchangeRef, JourneyPatternRef, LineDirection, LineRef, OperationalUnitRef, OperatorRef,
    PlaceRef, ProductCategoryRef, QuayRef, RequestedLines, RouteRef, ServiceFeatureRef, SituationFullRef,
    SituationNumber, SituationRef, StopAreaRef, StopPlaceComponentRef, StopPlaceRef, StopPointRef,
    VehicleFeatureRef, VehicleJourneyRef, VehicleRef, VersionRef,
};
