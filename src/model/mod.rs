//! Model elements shared by every SIRI service: references, positions, journeys.

mod accessibility;
mod call;
mod dated_journey;
mod estimated_journey;
mod facility;
mod feature;
mod formation;
mod interchange_journey;
mod journey;
mod location;
mod mode;
mod monitored_journey;
mod projection;
mod reference;
mod targeted_journey;

pub use accessibility::{
    AccessibilityAssessment, AccessibilityLimitation, AccessibilityLimitations,
    AssessmentSuitabilities, PassengerAccessibilityNeeds, Suitabilities, Suitability, Timeband,
    Timebands, UserNeed, UserNeedKind, ValidityCondition, ACSB_NAMESPACE, IFOPT_NAMESPACE,
};
pub use call::{
    BoardingPositionRef, FlexibleAreaRef, MaximumNumberOfCalls, PlannedStopAssignment,
    StandingPlace, StopAssignment,
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
pub use facility::{
    AnnotatedFacility, CountedAmount, CountedItemsIdList, EquipmentAvailability, EquipmentFeatures,
    Facility, FacilityChange, FacilityCondition, FacilityFeature, FacilityFeatureKind,
    FacilityFeatures, FacilityLimitations, FacilityLocation, FacilityStatus, FacilitySubject,
    MobilityDisruption, MonitoredCounting, MonitoringInformation, MonitoringValidityCondition,
    Remedy, TypeOfValue,
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
pub use interchange_journey::InterchangeJourney;
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
    BlockRef, ClearDownRef, ConnectionLinkRef, ControlActionRef, CourseOfJourneyRef, DataFrameRef,
    DatedVehicleJourneyRef, DestinationRef, DirectionRef, EquipmentRef, EquipmentTypeRef,
    FacilityRef, FeatureRef, FramedVehicleJourneyRef, InterchangeRef, JourneyPatternRef,
    LineDirection, LineRef, MonitoringRef, OperationalUnitRef, OperatorRef, OrganisationRef,
    PlaceRef, ProductCategoryRef, QuayRef, RequestedLines, RouteRef, ServiceFeatureRef, SiteRef,
    SituationFullRef, SituationNumber, SituationRef, StopAreaRef, StopPlaceComponentRef,
    StopPlaceRef, StopPointRef, VehicleFeatureRef, VehicleJourneyRef, VehicleRef, VersionRef,
};
pub use targeted_journey::{TargetedCall, TargetedVehicleJourney};
