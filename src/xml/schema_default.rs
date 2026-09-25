//! The value the schema gives an element that is written empty, declaration by
//! declaration.
//!
//! Generated from the schemas under `tests/fixtures/xsd` by `tests/schema_default.rs`;
//! after a schema change, regenerate it with
//! `UPDATE_SCHEMA_DEFAULT=1 cargo test --test schema_default`. Do not edit by hand: the test
//! fails when this file and the schemas disagree.
//!
//! XML Schema lets an element declaration carry a `default` or a `fixed` value. An
//! element so declared that is written with no content — `<Monitored/>`,
//! `<Monitored></Monitored>` — is valid and means the declared value. Each function
//! below reads one declared value that way, through
//! [`Defaulted`](super::defaulted::Defaulted), and a field carries the function of
//! the declaration it transcribes. What the value means for an element declared with
//! and without a default, and how it is written back, is said there.
//!
//! Attribute defaults are not listed: an attribute's default applies when the
//! attribute is absent, and an absent attribute reads as `None`.
//!
//! 279 declarations in 39 schema files, 28 distinct values.

use serde::Deserializer;

use super::defaulted::Defaulted;

/// The boolean `false`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `Advertised` | `xsd:boolean` | `siri_model/siri_datedVehicleJourney.xsd`, `InterchangePropertyGroup` |
/// | `AffectedOnly` | `xsd:boolean` | `siri_model/siri_situationAffects.xsd`, `StopPoints` |
/// | `AllData` | `xsd:boolean` | `siri/siri_common_services.xsd`, `DataSupplyTopicGroup` |
/// | `BoardingStretch` | `xsd:boolean` | `siri_model/siri_journey.xsd`, `CallPropertyGroup` |
/// | `CallForMeans` | `xsd:boolean` | `siri_controlAction_service.xsd`, `DriverMessageStructure` |
/// | `CallForRepairs` | `xsd:boolean` | `siri_controlAction_service.xsd`, `DriverMessageStructure` |
/// | `Cancellation` | `InterchangeCancellation` | `siri_model/siri_datedVehicleJourney.xsd`, `AbstractServiceJourneyInterchangeStructure` |
/// | `Cancellation` | `xsd:boolean` | `siri_model/siri_datedVehicleJourney.xsd`, `DatedCallStructure` |
/// | `Cancellation` | `InterchangeCancellation` | `siri_model/siri_datedVehicleJourney.xsd`, `ServiceJourneyInterchangeStructure` |
/// | `Cancellation` | `xsd:boolean` | `siri_model/siri_datedVehicleJourney.xsd`, `TimetableAlterationGroup` |
/// | `Cancellation` | `CallCancellation` | `siri_model/siri_estimatedVehicleJourney.xsd`, `EstimatedCallStructure` |
/// | `Cancellation` | `xsd:boolean` | `siri_model/siri_estimatedVehicleJourney.xsd`, `EstimatedTimetableAlterationGroup` |
/// | `Cancellation` | `CallCancellation` | `siri_model/siri_estimatedVehicleJourney.xsd`, `RecordedCallStructure` |
/// | `ConfirmDelivery` | `xsd:boolean` | `siri/siri_common_services.xsd`, `DeliveryContextGroup` |
/// | `DeAssignment` | `xsd:boolean` | `siri_controlAction_service.xsd`, `VehicleWorkAssignment` |
/// | `ExtraCall` | `xsd:boolean` | `siri_model/siri_datedVehicleJourney.xsd`, `DatedCallStructure` |
/// | `ExtraCall` | `xsd:boolean` | `siri_model/siri_estimatedVehicleJourney.xsd` |
/// | `ExtraInterchange` | `xsd:boolean` | `siri_model/siri_datedVehicleJourney.xsd` |
/// | `ExtraJourney` | `xsd:boolean` | `siri_model/siri_datedVehicleJourney.xsd`, `TimetableAlterationGroup` |
/// | `ExtraJourney` | `xsd:boolean` | `siri_model/siri_estimatedVehicleJourney.xsd`, `EstimatedTimetableAlterationGroup` |
/// | `FilterByDestination` | `xsd:boolean` | `siri_model/siri_modelPermissions.xsd` |
/// | `FilterByInterchangeRef` | `xsd:boolean` | `siri_model/siri_modelPermissions.xsd` |
/// | `FilterByVehicleJourneyRef` | `xsd:boolean` | `siri_model/siri_modelPermissions.xsd` |
/// | `FilterByVehicleRef` | `xsd:boolean` | `siri_model/siri_modelPermissions.xsd` |
/// | `ForeignJourneysOnly` | `xsd:boolean` | `siri_connectionMonitoring_service.xsd`, `RequestPolicy` |
/// | `ForeignJourneysOnly` | `xsd:boolean` | `siri_connectionTimetable_service.xsd`, `RequestPolicy` |
/// | `Guaranteed` | `xsd:boolean` | `siri_model/siri_datedVehicleJourney.xsd`, `InterchangePropertyGroup` |
/// | `HasConfirmDelivery` | `xsd:boolean` | `siri/siri_requests.xsd`, `CapabilityGeneralInteractionStructure` |
/// | `HasDetailLevel` | `xsd:boolean` | `siri_stopMonitoring_service.xsd`, `StopMonitoringVolumeGroup` |
/// | `HasDetailLevel` | `xsd:boolean` | `siri_vehicleMonitoring_service.xsd`, `VehicleMonitoringVolumeGroup` |
/// | `HasDriverMessages` | `xsd:boolean` | `siri_controlAction_service.xsd`, `ControlActionCapabilityRequestPolicyStructure` |
/// | `HasHeartbeat` | `xsd:boolean` | `siri/siri_requests.xsd`, `CapabilityGeneralInteractionStructure` |
/// | `HasMaximumFacilityStatus` | `xsd:boolean` | `siri_facilityMonitoring_service.xsd`, `FacilityMonitoringVolumeGroup` |
/// | `HasMaximumNumberOfCalls` | `xsd:boolean` | `siri_vehicleMonitoring_service.xsd`, `VehicleMonitoringVolumeGroup` |
/// | `HasMaximumNumberOfSituations` | `xsd:boolean` | `siri_situationExchange_service.xsd`, `SituationExchangeVolumeGroup` |
/// | `HasMinimumVisitsPerVia` | `xsd:boolean` | `siri_stopMonitoring_service.xsd`, `StopMonitoringVolumeGroup` |
/// | `HasNames` | `xsd:boolean` | `siri_vehicleMonitoring_service.xsd`, `VehicleMonitoringCapabilityRequestPolicyStructure` |
/// | `HasNumberOfOnwardsCalls` | `xsd:boolean` | `siri_stopMonitoring_service.xsd`, `StopMonitoringVolumeGroup` |
/// | `HasNumberOfOnwardsCalls` | `xsd:boolean` | `siri_vehicleMonitoring_service.xsd`, `VehicleMonitoringVolumeGroup` |
/// | `HasNumberOfPreviousCalls` | `xsd:boolean` | `siri_stopMonitoring_service.xsd`, `StopMonitoringVolumeGroup` |
/// | `HasNumberOfPreviousCalls` | `xsd:boolean` | `siri_vehicleMonitoring_service.xsd`, `VehicleMonitoringVolumeGroup` |
/// | `HasRemedy` | `xsd:boolean` | `siri_facilityMonitoring_service.xsd`, `ResponseFeatures` |
/// | `HasSituations` | `xsd:boolean` | `siri_controlAction_service.xsd`, `ResponseFeatures` |
/// | `HasSituations` | `xsd:boolean` | `siri_stopMonitoring_service.xsd`, `ResponseFeatures` |
/// | `HasSituations` | `xsd:boolean` | `siri_vehicleMonitoring_service.xsd`, `ResponseFeatures` |
/// | `HasVehicleDetectings` | `xsd:boolean` | `siri_controlAction_service.xsd`, `ControlActionCapabilityRequestPolicyStructure` |
/// | `HeadwayService` | `xsd:boolean` | `siri_model/siri_journey.xsd`, `JourneyEndTimesGroup` |
/// | `HeadwayService` | `xsd:boolean` | `siri_model/siri_situationAffects.xsd`, `AffectedVehicleStructure` |
/// | `HomePage` | `xsd:boolean` | `siri_model/siri_situationActions.xsd`, `PublishToMobileActionStructure` |
/// | `HomePage` | `xsd:boolean` | `siri_model/siri_situationActions.xsd`, `PublishToWebActionStructure` |
/// | `InPanic` | `xsd:boolean` | `siri_model/siri_monitoredVehicleJourney.xsd`, `ProgressDataQualityGroup` |
/// | `InPanic` | `xsd:boolean` | `siri_model/siri_situationAffects.xsd`, `AffectedVehicleStructure` |
/// | `IncludeDriverMessages` | `xsd:boolean` | `siri_controlAction_service.xsd`, `ControlActionTopicGroup` |
/// | `IncludeOnlyIfInPublicationWindow` | `xsd:boolean` | `siri_situationExchange_service.xsd`, `TemporalContentFilterGroup` |
/// | `IncludeOnlyRecordedCallUpdates` | `xsd:boolean` | `siri_estimatedTimetable_service.xsd`, `EstimatedTimetableSubscriptionPolicyGroup` |
/// | `IncludeSituations` | `xsd:boolean` | `siri_controlAction_service.xsd`, `ControlActionRequestPolicyGroup` |
/// | `IncludeSituations` | `xsd:boolean` | `siri_stopMonitoring_service.xsd`, `StopMonitoringRequestPolicyGroup` |
/// | `IncludeTranslations` | `xsd:boolean` | `siri/siri_request_support.xsd` |
/// | `IncludeVehicleDetectings` | `xsd:boolean` | `siri_controlAction_service.xsd`, `ControlActionTopicGroup` |
/// | `IncrementalUpdates` | `xsd:boolean` | `siri_controlAction_service.xsd`, `ControlActionSubscriptionPolicyGroup` |
/// | `IncrementalUpdates` | `xsd:boolean` | `siri_facilityMonitoring_service.xsd`, `FacilityMonitoringSubscriptionPolicyGroup` |
/// | `IncrementalUpdates` | `xsd:boolean` | `siri_productionTimetable_service.xsd`, `ProductionTimetableRequestPolicyGroup` |
/// | `IncrementalUpdates` | `xsd:boolean` | `siri_productionTimetable_service.xsd`, `ProductionTimetableSubscriptionPolicyGroup` |
/// | `IncrementalUpdates` | `xsd:boolean` | `siri_situationExchange_service.xsd`, `SituationExchangeSubscriptionPolicyGroup` |
/// | `IncrementalUpdates` | `xsd:boolean` | `siri_stopMonitoring_service.xsd`, `StopMonitoringSubscriptionPolicyGroup` |
/// | `IncrementalUpdates` | `xsd:boolean` | `siri_stopTimetable_service.xsd`, `StopTimetableSubscriptionPolicyGroup` |
/// | `IncrementalUpdates` | `xsd:boolean` | `siri_vehicleMonitoring_service.xsd`, `VehicleMonitoringSubscriptionPolicyGroup` |
/// | `IsCompleteStopSequence` | `xsd:boolean` | `siri_model/siri_estimatedVehicleJourney.xsd`, `EstimatedVehicleJourneyStructure` |
/// | `IsCompleteStopSequence` | `xsd:boolean` | `siri_model/siri_monitoredVehicleJourney.xsd`, `MonitoredCallingPatternGroup` |
/// | `JourneyPlanner` | `xsd:boolean` | `siri_model/siri_situation.xsd`, `BlockingStructure` |
/// | `MoreData` | `xsd:boolean` | `siri.xsd`, `ServiceDeliveryBodyGroup` |
/// | `MoreData` | `xsd:boolean` | `siri/siri_base.xsd`, `ServiceDeliveryBodyGroup` |
/// | `MultipleSubscriberFilter` | `xsd:boolean` | `siri/siri_requests.xsd`, `CapabilityGeneralInteractionStructure` |
/// | `OnBoard` | `xsd:boolean` | `siri_model/siri_situationActions.xsd`, `PublishToDisplayActionStructure` |
/// | `OriginatedByDriver` | `xsd:boolean` | `siri_controlAction_service.xsd`, `DriverMessageStructure` |
/// | `ParticipantPermissions` | `xsd:boolean` | `siri/siri_requests.xsd`, `ServiceCapabilitiesRequestStructure` |
/// | `Planned` | `xsd:boolean` | `siri_model/siri_situation.xsd`, `ClassifierGroup` |
/// | `PlatformTraversal` | `xsd:boolean` | `siri_model/siri_monitoredVehicleJourney.xsd`, `CallRailGroup` |
/// | `PredictionInaccurate` | `xsd:boolean` | `siri_model/siri_journey.xsd` |
/// | `Premium` | `xsd:boolean` | `siri_model/siri_situationActions.xsd`, `NotifyBySmsActionStructure` |
/// | `RealTime` | `xsd:boolean` | `siri_model/siri_situation.xsd`, `BlockingStructure` |
/// | `RequestChecking` | `xsd:boolean` | `siri_utility/siri_permissions.xsd`, `CapabilityAccessControlStructure` |
/// | `RequestStop` | `xsd:boolean` | `siri_model/siri_journey.xsd`, `CallPropertyGroup` |
/// | `ReversedOrientation` | `xsd:boolean` | `siri_model/siri_journey.xsd`, `TrainComponentGroup` |
/// | `ReversedOrientation` | `xsd:boolean` | `siri_model/siri_journey.xsd`, `TrainInCompoundTrainGroup` |
/// | `ReversesAtStop` | `xsd:boolean` | `siri_model/siri_monitoredVehicleJourney.xsd`, `CallRailGroup` |
/// | `SkipRecordedCallUpdates` | `xsd:boolean` | `siri_estimatedTimetable_service.xsd`, `EstimatedTimetableSubscriptionPolicyGroup` |
/// | `StaySeated` | `xsd:boolean` | `siri_controlAction_service.xsd`, `ExtraConnection` |
/// | `StaySeated` | `xsd:boolean` | `siri_model/siri_datedVehicleJourney.xsd`, `InterchangePropertyGroup` |
/// | `Ticker` | `xsd:boolean` | `siri_model/siri_situationActions.xsd`, `PublishToWebActionStructure` |
/// | `UseNames` | `xsd:boolean` | `siri_stopMonitoring_service.xsd`, `StopMonitoringCapabilityRequestPolicyStructure` |
/// | `UseNames` | `xsd:boolean` | `siri_stopTimetable_service.xsd`, `StopTimetableCapabilityRequestPolicyStructure` |
/// | `VehicleAtStop` | `xsd:boolean` | `siri_model/siri_reference.xsd` |
/// | `VisitNumberisOrder` | `xsd:boolean` | `siri/siri_requests.xsd`, `CapabilityGeneralInteractionStructure` |
pub(crate) fn boolean_false<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "false")
}

/// The boolean `true`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `Allow` | `xsd:boolean` | `siri_utility/siri_permissions.xsd`, `AbstractTopicPermissionStructure` |
/// | `ByEmail` | `xsd:boolean` | `siri_model/siri_situationActions.xsd`, `PublishToAlertsActionStructure` |
/// | `ByMobile` | `xsd:boolean` | `siri_model/siri_situationActions.xsd`, `PublishToAlertsActionStructure` |
/// | `ByStartTime` | `xsd:boolean` | `siri_controlAction_service.xsd`, `TopicFiltering` |
/// | `ByStartTime` | `xsd:boolean` | `siri_stopMonitoring_service.xsd`, `TopicFiltering` |
/// | `Ceefax` | `xsd:boolean` | `siri_model/siri_situationActions.xsd`, `PublishToTvActionStructure` |
/// | `CheckConnectionLinkRef` | `xsd:boolean` | `siri_model/siri_modelPermissions.xsd` |
/// | `CheckInfoChannelRef` | `xsd:boolean` | `siri_generalMessage_service.xsd`, `GeneralMessageCapabilityAccessControlStructure` |
/// | `CheckLineRef` | `xsd:boolean` | `siri_model/siri_modelPermissions.xsd` |
/// | `CheckMonitoringRef` | `xsd:boolean` | `siri_model/siri_modelPermissions.xsd` |
/// | `CheckOperatorRef` | `xsd:boolean` | `siri_model/siri_modelPermissions.xsd` |
/// | `CheckVehicleMonitoringRef` | `xsd:boolean` | `siri_vehicleMonitoring_service.xsd`, `AccessControl` |
/// | `ConcernsArrivals` | `xsd:boolean` | `siri_controlAction_service.xsd`, `CallCancellationActionStructure` |
/// | `ConcernsDepartures` | `xsd:boolean` | `siri_controlAction_service.xsd`, `CallCancellationActionStructure` |
/// | `EngineOn` | `xsd:boolean` | `siri_model/siri_monitoredVehicleJourney.xsd`, `ProgressDataGroup` |
/// | `FilterByConnectionLinkRef` | `xsd:boolean` | `siri_model/siri_modelPermissions.xsd` |
/// | `FilterByDirectionRef` | `xsd:boolean` | `siri_model/siri_modelPermissions.xsd` |
/// | `FilterByFacilityRef` | `xsd:boolean` | `siri_model/siri_modelPermissions.xsd` |
/// | `FilterByInfoChannel` | `xsd:boolean` | `siri_generalMessage_service.xsd`, `TopicFiltering` |
/// | `FilterByJourney` | `xsd:boolean` | `siri_connectionMonitoring_service.xsd`, `TopicFiltering` (fixed) |
/// | `FilterByKeyword` | `xsd:boolean` | `siri_situationExchange_service.xsd`, `TopicFiltering` |
/// | `FilterByLineRef` | `xsd:boolean` | `siri_model/siri_modelPermissions.xsd` |
/// | `FilterByLocationRef` | `xsd:boolean` | `siri_facilityMonitoring_service.xsd`, `TopicFiltering` (fixed) |
/// | `FilterByLocationRef` | `xsd:boolean` | `siri_situationExchange_service.xsd`, `TopicFiltering` (fixed) |
/// | `FilterByMode` | `xsd:boolean` | `siri_controlAction_service.xsd`, `TopicFiltering` |
/// | `FilterByMonitoringRef` | `xsd:boolean` | `siri_model/siri_modelPermissions.xsd` (fixed) |
/// | `FilterByNetworkRef` | `xsd:boolean` | `siri_controlAction_service.xsd`, `TopicFiltering` |
/// | `FilterByOperatorRef` | `xsd:boolean` | `siri_model/siri_modelPermissions.xsd` |
/// | `FilterByProductCategoryRef` | `xsd:boolean` | `siri_model/siri_modelPermissions.xsd` |
/// | `FilterBySpecificNeed` | `xsd:boolean` | `siri_facilityMonitoring_service.xsd`, `TopicFiltering` |
/// | `FilterBySpecificNeed` | `xsd:boolean` | `siri_situationExchange_service.xsd`, `TopicFiltering` |
/// | `FilterByStopPointRef` | `xsd:boolean` | `siri_model/siri_modelPermissions.xsd` |
/// | `FilterByTime` | `xsd:boolean` | `siri_connectionMonitoring_service.xsd`, `TopicFiltering` |
/// | `FilterByValidityPeriod` | `xsd:boolean` | `siri_model/siri_modelPermissions.xsd` |
/// | `FilterByVehicleMode` | `xsd:boolean` | `siri_model/siri_modelPermissions.xsd` |
/// | `FilterByVehicleMonitoringRef` | `xsd:boolean` | `siri_vehicleMonitoring_service.xsd`, `TopicFiltering` (fixed) |
/// | `FilterByVersionRef` | `xsd:boolean` | `siri_estimatedTimetable_service.xsd`, `TopicFiltering` |
/// | `FilterByVersionRef` | `xsd:boolean` | `siri_productionTimetable_service.xsd`, `TopicFiltering` |
/// | `FilterByVisitType` | `xsd:boolean` | `siri_stopMonitoring_service.xsd`, `TopicFiltering` |
/// | `HasChangeSensitivity` | `xsd:boolean` | `siri/siri_requests.xsd`, `CapabilitySubscriptionPolicyStructure` |
/// | `HasFacilityLocation` | `xsd:boolean` | `siri_facilityMonitoring_service.xsd`, `ResponseFeatures` |
/// | `HasIncrementalUpdates` | `xsd:boolean` | `siri/siri_requests.xsd`, `CapabilitySubscriptionPolicyStructure` |
/// | `HasIncrementalUpdates` | `xsd:boolean` | `siri_productionTimetable_service.xsd`, `SubscriptionPolicy` |
/// | `HasLineNotices` | `xsd:boolean` | `siri_stopMonitoring_service.xsd`, `ResponseFeatures` |
/// | `HasLocation` | `xsd:boolean` | `siri_vehicleMonitoring_service.xsd`, `ResponseFeatures` |
/// | `HasMaximumControlActions` | `xsd:boolean` | `siri_controlAction_service.xsd`, `ControlActionVolumeGroup` |
/// | `HasMaximumVehicles` | `xsd:boolean` | `siri_vehicleMonitoring_service.xsd`, `VehicleMonitoringVolumeGroup` |
/// | `HasMaximumVisits` | `xsd:boolean` | `siri_stopMonitoring_service.xsd`, `StopMonitoringVolumeGroup` |
/// | `HasMinimumVisitsPerLine` | `xsd:boolean` | `siri_stopMonitoring_service.xsd`, `StopMonitoringVolumeGroup` |
/// | `Incidents` | `xsd:boolean` | `siri_model/siri_situationActions.xsd`, `PublishToMobileActionStructure` |
/// | `Incidents` | `xsd:boolean` | `siri_model/siri_situationActions.xsd`, `PublishToWebActionStructure` |
/// | `IncludeInterchanges` | `xsd:boolean` | `siri/siri_request_support.xsd` |
/// | `IncludeJourneyRelations` | `xsd:boolean` | `siri/siri_request_support.xsd` |
/// | `IncludeTrainFormations` | `xsd:boolean` | `siri/siri_request_support.xsd` |
/// | `IncrementalUpdates` | `xsd:boolean` | `siri_estimatedTimetable_service.xsd`, `EstimatedTimetableSubscriptionPolicyGroup` |
/// | `IsExposedToPassengers` | `xsd:boolean` | `siri_controlAction_service.xsd`, `ExtraConnection` |
/// | `IsExposedToStaff` | `xsd:boolean` | `siri_controlAction_service.xsd`, `ExtraConnection` |
/// | `Monitored` | `xsd:boolean` | `siri_model/siri_datedVehicleJourney.xsd`, `TimetableRealtimeInfoGroup` |
/// | `Monitored` | `xsd:boolean` | `siri_model/siri_facility.xsd`, `AnnotatedFacilityStructure` |
/// | `Monitored` | `xsd:boolean` | `siri_model/siri_interchangeJourney.xsd`, `InterchangeJourneyStructure` |
/// | `Monitored` | `xsd:boolean` | `siri_model/siri_monitoredVehicleJourney.xsd`, `JourneyProgressGroup` |
/// | `Monitored` | `xsd:boolean` | `siri_model_discovery/siri_connectionLink.xsd`, `AnnotatedConnectionLinkStructure` |
/// | `Monitored` | `xsd:boolean` | `siri_model_discovery/siri_line.xsd`, `AnnotatedLineStructure` |
/// | `Monitored` | `xsd:boolean` | `siri_model_discovery/siri_stopPoint.xsd`, `AnnotatedStopPointStructure` |
/// | `MultipartDespatch` | `xsd:boolean` | `siri/siri_requests.xsd`, `CapabilityGeneralInteractionStructure` |
/// | `OnPlace` | `xsd:boolean` | `siri_model/siri_situationActions.xsd`, `PublishToDisplayActionStructure` |
/// | `PublishSubscribe` | `xsd:boolean` | `siri/siri_requests.xsd`, `Interaction` |
/// | `PublishSubscribe` | `xsd:boolean` | `siri_utility/siri_permissions.xsd`, `GeneralCapabilities` |
/// | `RequestResponse` | `xsd:boolean` | `siri/siri_requests.xsd`, `Interaction` |
/// | `RequestResponse` | `xsd:boolean` | `siri_utility/siri_permissions.xsd`, `GeneralCapabilities` |
/// | `ReversingDirection` | `xsd:boolean` | `siri_model/siri_journey.xsd`, `VehicleTypePropertiesGroup` |
/// | `SelfPropelled` | `xsd:boolean` | `siri_model/siri_journey.xsd`, `VehicleTypePropertiesGroup` |
/// | `Status` | `xsd:boolean` | `siri/siri_requests.xsd` |
/// | `Status` | `xsd:boolean` | `siri/siri_requests.xsd`, `ServiceDeliveryRequestStatusGroup` |
/// | `Teletext` | `xsd:boolean` | `siri_model/siri_situationActions.xsd`, `PublishToTvActionStructure` |
/// | `TimingPoint` | `xsd:boolean` | `siri_model/siri_reference.xsd` |
/// | `UseReferences` | `xsd:boolean` | `siri_stopMonitoring_service.xsd`, `StopMonitoringCapabilityRequestPolicyStructure` |
/// | `UseReferences` | `xsd:boolean` | `siri_stopTimetable_service.xsd`, `StopTimetableCapabilityRequestPolicyStructure` |
pub(crate) fn boolean_true<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "true")
}

/// The integer `2`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `ViaPriority` | `xsd:nonNegativeInteger` | `siri_model/siri_journey.xsd`, `ViaNameStructure` |
pub(crate) fn integer_2<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "2")
}

/// The integer `30`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `MaximumTextLength` | `xsd:positiveInteger` | `siri_stopMonitoring_service.xsd`, `StopMonitoringRequestPolicyGroup` |
pub(crate) fn integer_30<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "30")
}

/// The decimal `0.9`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `Percentile` | `xsd:decimal` | `siri_model/siri_journey.xsd`, `PredictionQualityStructure` |
pub(crate) fn decimal_0_9<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "0.9")
}

/// The duration `PT60M`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `DefaultPreviewInterval` | `PositiveDurationType` | `siri_controlAction_service.xsd`, `TopicFiltering` |
/// | `DefaultPreviewInterval` | `PositiveDurationType` | `siri_facilityMonitoring_service.xsd`, `TopicFiltering` |
/// | `DefaultPreviewInterval` | `PositiveDurationType` | `siri_generalMessage_service.xsd`, `TopicFiltering` |
/// | `DefaultPreviewInterval` | `PositiveDurationType` | `siri_situationExchange_service.xsd`, `TopicFiltering` |
/// | `DefaultPreviewInterval` | `PositiveDurationType` | `siri_stopMonitoring_service.xsd`, `TopicFiltering` |
/// | `DefaultPreviewInterval` | `PositiveDurationType` | `siri_vehicleMonitoring_service.xsd`, `TopicFiltering` |
pub(crate) fn duration_pt60m<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "PT60M")
}

/// The string `en`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `Language` | `xsd:language` | `siri/siri_common_services.xsd`, `ReferenceContextGroup` |
/// | `Language` | `xsd:language` | `siri_connectionMonitoring_service.xsd`, `ConnectionMonitoringRequestPolicyGroup` |
/// | `Language` | `xsd:language` | `siri_connectionTimetable_service.xsd`, `ConnectionTimetableRequestPolicyGroup` |
/// | `Language` | `xsd:language` | `siri_controlAction_service.xsd`, `ControlActionRequestPolicyGroup` |
/// | `Language` | `xsd:language` | `siri_discovery.xsd`, `ConnectionLinksDiscoveryRequestPolicyGroup` |
/// | `Language` | `xsd:language` | `siri_discovery.xsd`, `FacilityRequestPolicyGroup` |
/// | `Language` | `xsd:language` | `siri_discovery.xsd`, `LinesDiscoveryRequestPolicyGroup` |
/// | `Language` | `xsd:language` | `siri_discovery.xsd`, `ProductCategoriesRequestPolicyGroup` |
/// | `Language` | `xsd:language` | `siri_discovery.xsd`, `StopPointsDiscoveryRequestPolicyGroup` |
/// | `Language` | `xsd:language` | `siri_discovery.xsd`, `VehicleFeaturesRequestPolicyGroup` |
/// | `Language` | `xsd:language` | `siri_estimatedTimetable_service.xsd`, `EstimatedTimetableRequestPolicyGroup` |
/// | `Language` | `xsd:language` | `siri_facilityMonitoring_service.xsd`, `FacilityMonitoringRequestPolicyGroup` |
/// | `Language` | `xsd:language` | `siri_generalMessage_service.xsd`, `GeneralMessageRequestPolicyGroup` |
/// | `Language` | `xsd:language` | `siri_model/siri_situation.xsd`, `DescriptionGroup` |
/// | `Language` | `xsd:language` | `siri_productionTimetable_service.xsd`, `ProductionTimetableRequestPolicyGroup` |
/// | `Language` | `xsd:language` | `siri_situationExchange_service.xsd`, `SituationExchangeRequestPolicyGroup` |
/// | `Language` | `xsd:language` | `siri_stopMonitoring_service.xsd`, `StopMonitoringRequestPolicyGroup` |
/// | `Language` | `xsd:language` | `siri_stopTimetable_service.xsd`, `StopTimetableRequestPolicyGroup` |
/// | `Language` | `xsd:language` | `siri_vehicleMonitoring_service.xsd`, `VehicleMonitoringRequestPolicyGroup` |
pub(crate) fn string_en<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "en")
}

/// The enumeration `alighting`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `ArrivalBoardingActivity` | `ArrivalBoardingActivityEnumeration` | `siri_model/siri_journey.xsd` |
/// | `ArrivalBoardingActivity` | `ArrivalBoardingActivityEnumeration` | `siri_model/siri_situation.xsd`, `BoardingStructure` |
pub(crate) fn enumeration_alighting<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "alighting")
}

/// The enumeration `all`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `StopVisitTypes` | `StopVisitTypeEnumeration` | `siri_stopMonitoring_service.xsd`, `StopMonitoringTopicGroup` |
pub(crate) fn enumeration_all<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "all")
}

/// The enumeration `anyone`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `AllowedPredictors` | `PredictorsEnumeration` | `siri/siri_common_services.xsd`, `PredictionMethodGroup` |
pub(crate) fn enumeration_anyone<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "anyone")
}

/// The enumeration `boarding`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `DepartureBoardingActivity` | `DepartureBoardingActivityEnumeration` | `siri_model/siri_journey.xsd` |
/// | `DepartureBoardingActivity` | `DepartureBoardingActivityEnumeration` | `siri_model/siri_situation.xsd`, `BoardingStructure` |
pub(crate) fn enumeration_boarding<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "boarding")
}

/// The enumeration `both`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `ConnectionDirection` | `ConnectionDirectionEnumeration` | `siri_model/siri_situationAffects.xsd`, `AffectedConnectionLinkStructure` |
pub(crate) fn enumeration_both<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "both")
}

/// The enumeration `calls`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `EstimatedTimetableDetailLevel` | `EstimatedTimetableDetailEnumeration` | `siri_estimatedTimetable_service.xsd`, `EstimatedTimetableRequestPolicyGroup` |
pub(crate) fn enumeration_calls<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "calls")
}

/// The enumeration `direct`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `DeliveryMethod` | `DeliveryMethodEnumeration` | `siri/siri_common_services.xsd`, `DeliveryContextGroup` |
pub(crate) fn enumeration_direct<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "direct")
}

/// The enumeration `everyDay`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `DayType` | `DayTypeEnumeration` | `siri_model/siri_time.xsd` |
pub(crate) fn enumeration_every_day<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "everyDay")
}

/// The enumeration `false`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `AudibleSignalsAvailable` | `AccessibilityStructure` | `acsb/acsb_limitations.xsd` |
/// | `WheelchairAccess` | `AccessibilityStructure` | `acsb/acsb_limitations.xsd` |
// No field transcribes a declaration with this value; the function is kept so
// that the table is the schema's rather than the crate's.
#[allow(dead_code)]
pub(crate) fn enumeration_false<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "false")
}

/// The enumeration `httpPost`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `CommunicationsTransportMethod` | `CommunicationsTransportMethodEnumeration` | `siri/siri_requests.xsd`, `TransportDescriptionStructure` |
pub(crate) fn enumeration_http_post<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "httpPost")
}

/// The enumeration `minimal`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `ChangeModel` | `ChangeModelEnumeration` | `siri_controlAction_service.xsd`, `RelativeTime` |
pub(crate) fn enumeration_minimal<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "minimal")
}

/// The enumeration `none`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `CompressionMethod` | `CompressionMethodEnumeration` | `siri/siri_requests.xsd`, `TransportDescriptionStructure` |
pub(crate) fn enumeration_none<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "none")
}

/// The enumeration `normal`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `ConnectionLinksDetailLevel` | `ConnectionLinksDetailEnumeration` | `siri_discovery.xsd`, `ConnectionLinksDiscoveryRequestPolicyGroup` |
/// | `ConnectionMonitoringDetailLevel` | `ConnectionMonitoringDetailEnumeration` | `siri_connectionMonitoring_service.xsd`, `ConnectionMonitoringRequestPolicyGroup` |
/// | `DefaultDetailLevel` | `StopMonitoringDetailEnumeration` | `siri_stopMonitoring_service.xsd`, `StopMonitoringVolumeGroup` |
/// | `DefaultDetailLevel` | `VehicleMonitoringDetailEnumeration` | `siri_vehicleMonitoring_service.xsd`, `VehicleMonitoringVolumeGroup` |
/// | `FacilityDetailLevel` | `FacilityDetailEnumeration` | `siri_discovery.xsd`, `FacilityRequestPolicyGroup` |
/// | `LinesDetailLevel` | `LinesDetailEnumeration` | `siri_discovery.xsd`, `LinesDiscoveryRequestPolicyGroup` |
/// | `Severity` | `SeverityEnumeration` | `siri_model/siri_situationClassifiers.xsd` |
/// | `Severity` | `SeverityEnumeration` | `siri_situationExchange_service.xsd`, `SituationClassifierFilterGroup` |
/// | `StopMonitoringDetailLevel` | `StopMonitoringDetailEnumeration` | `siri_stopMonitoring_service.xsd`, `StopMonitoringRequestPolicyGroup` |
/// | `StopPointsDetailLevel` | `StopPointsDetailEnumeration` | `siri_discovery.xsd`, `StopPointsDiscoveryRequestPolicyGroup` |
/// | `TrainSizeType` | `TrainSizeEnumeration` | `siri_model/siri_journey.xsd`, `TrainGroup` |
pub(crate) fn enumeration_normal<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "normal")
}

/// The enumeration `notAvailable`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `EquipmentStatus` | `ifopt:EquipmentStatusEnumeration` | `siri_model/siri_facility.xsd`, `EquipmentAvailabilityStructure` |
pub(crate) fn enumeration_not_available<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "notAvailable")
}

/// The enumeration `open`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `ActionStatus` | `ActionStatusEnumeration` | `siri_model/siri_situationActions.xsd`, `SimpleActionStructure` |
/// | `Progress` | `WorkflowStatusEnumeration` | `siri_model/siri_situation.xsd`, `StatusGroup` |
/// | `Progress` | `WorkflowStatusEnumeration` | `siri_situationExchange_service.xsd`, `SituationStatusFilterGroup` |
pub(crate) fn enumeration_open<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "open")
}

/// The enumeration `public`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `Audience` | `AudienceEnumeration` | `siri_model/siri_situation.xsd`, `ClassifierGroup` |
pub(crate) fn enumeration_public<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "public")
}

/// The enumeration `reliable`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `ConfidenceLevel` | `QualityIndexEnumeration` | `siri_model/siri_monitoredVehicleJourney.xsd`, `ProgressDataQualityGroup` |
/// | `QualityIndex` | `QualityIndexEnumeration` | `siri_model/siri_situation.xsd`, `StatusGroup` |
pub(crate) fn enumeration_reliable<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "reliable")
}

/// The enumeration `second`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `EndTimePrecision` | `EndTimePrecisionEnumeration` | `siri_model/siri_time.xsd`, `ClosedTimeRangeStructure` |
/// | `EndTimePrecision` | `EndTimePrecisionEnumeration` | `siri_model/siri_time.xsd`, `HalfOpenTimestampInputRangeStructure` |
pub(crate) fn enumeration_second<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "second")
}

/// The enumeration `undefined`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `EndTimeStatus` | `EndTimeStatusEnumeration` | `siri_model/siri_time.xsd`, `HalfOpenTimestampOutputRangeStructure` |
pub(crate) fn enumeration_undefined<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "undefined")
}

/// The enumeration `unknown`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `AccessFacility` | `AccessFacilityEnumeration` | `siri_model/siri_facilities.xsd` |
/// | `AccessFacility` | `AccessFacilityEnumeration` | `siri_model/siri_facilities.xsd`, `CommonFacilityGroup` |
/// | `AccommodationFacility` | `AccommodationFacilityEnumeration` | `siri_model/siri_facilities.xsd` |
/// | `AirSubmode` | `AirSubmodesOfTransportEnumeration` | `siri_model/siri_modes.xsd` |
/// | `AssistanceFacility` | `AssistanceFacilityEnumeration` | `siri_model/siri_facilities.xsd` |
/// | `BookingStatusType` | `BookingStatusEnumeration` | `siri_model/siri_situationServiceTypes.xsd` |
/// | `BusSubmode` | `BusSubmodesOfTransportEnumeration` | `siri_model/siri_modes.xsd` |
/// | `CoachSubmode` | `CoachSubmodesOfTransportEnumeration` | `siri_model/siri_modes.xsd` |
/// | `EscalatorFreeAccess` | `AccessibilityStructure` | `acsb/acsb_limitations.xsd` |
/// | `FareClass` | `FareClassEnumeration` | `siri_model/siri_journey_support.xsd` |
/// | `FareClassFacility` | `FareClassFacilityEnumeration` | `siri_model/siri_facilities.xsd` |
/// | `FunicularSubmode` | `FunicularSubmodesOfTransportEnumeration` | `siri_model/siri_modes.xsd` |
/// | `GuideDogAccess` | `AccessibilityStructure` | `acsb/acsb_limitations.xsd` |
/// | `HireFacility` | `HireFacilityEnumeration` | `siri_model/siri_facilities.xsd` |
/// | `InterchangeStatusType` | `InterchangeStatusEnumeration` | `siri_model/siri_situationServiceTypes.xsd` |
/// | `LiftFreeAccess` | `AccessibilityStructure` | `acsb/acsb_limitations.xsd` |
/// | `LuggageFacility` | `LuggageFacilityEnumeration` | `siri_model/siri_facilities.xsd` |
/// | `MetroSubmode` | `MetroSubmodesOfTransportEnumeration` | `siri_model/siri_modes.xsd` |
/// | `MobilityFacility` | `MobilityFacilityEnumeration` | `siri_model/siri_facilities.xsd` |
/// | `NuisanceFacility` | `NuisanceFacilityEnumeration` | `siri_model/siri_facilities.xsd` |
/// | `PassengerCommsFacility` | `PassengerCommsFacilityEnumeration` | `siri_model/siri_facilities.xsd` |
/// | `PassengerInformationFacility` | `PassengerInformationFacilityEnumeration` | `siri_model/siri_facilities.xsd` |
/// | `RailSubmode` | `RailSubmodesOfTransportEnumeration` | `siri_model/siri_modes.xsd` |
/// | `RefreshmentFacility` | `RefreshmentFacilityEnumeration` | `siri_model/siri_facilities.xsd` |
/// | `ReportType` | `ReportTypeEnumeration` | `siri_model/siri_situationServiceTypes.xsd` |
/// | `ReservedSpaceFacility` | `ReservedSpaceFacilityEnumeration` | `siri_model/siri_facilities.xsd` |
/// | `RetailFacility` | `RetailFacilityEnumeration` | `siri_model/siri_facilities.xsd` |
/// | `RoutePointType` | `RoutePointTypeEnumeration` | `siri_model/siri_situationServiceTypes.xsd` |
/// | `SanitaryFacility` | `SanitaryFacilityEnumeration` | `siri_model/siri_facilities.xsd` |
/// | `SelfDriveSubmode` | `SelfDriveSubmodesOfTransportEnumeration` | `siri_model/siri_modes.xsd` |
/// | `StepFreeAccess` | `AccessibilityStructure` | `acsb/acsb_limitations.xsd` |
/// | `StopPointType` | `StopPointTypeEnumeration` | `siri_model/siri_situationServiceTypes.xsd` |
/// | `TaxiSubmode` | `TaxiSubmodesOfTransportEnumeration` | `siri_model/siri_modes.xsd` |
/// | `TelecabinSubmode` | `TelecabinSubmodesOfTransportEnumeration` | `siri_model/siri_modes.xsd` |
/// | `TicketRestrictionType` | `TicketRestrictionEnumeration` | `siri_model/siri_situationServiceTypes.xsd` |
/// | `TicketingFacility` | `TicketingFacilityEnumeration` | `siri_model/siri_facilities.xsd` |
/// | `TimetableType` | `TimetableTypeEnumeration` | `siri_model/siri_situationServiceTypes.xsd` |
/// | `TramSubmode` | `TramSubmodesOfTransportEnumeration` | `siri_model/siri_modes.xsd` |
/// | `VehicleMode` | `VehicleModesOfTransportEnumeration` | `siri_model/siri_modes.xsd` |
/// | `VisualSignsAvailable` | `AccessibilityStructure` | `acsb/acsb_limitations.xsd` |
/// | `WaterSubmode` | `WaterSubmodesOfTransportEnumeration` | `siri_model/siri_modes.xsd` |
pub(crate) fn enumeration_unknown<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "unknown")
}

/// The enumeration `unspecified`, which the schema declares for:
///
/// | Element | Type | Declared in |
/// |---|---|---|
/// | `FirstOrLastJourney` | `FirstOrLastJourneyEnumeration` | `siri_model/siri_datedVehicleJourney.xsd`, `DatedServiceInfoGroup` |
/// | `FirstOrLastJourney` | `FirstOrLastJourneyEnumeration` | `siri_model/siri_journey.xsd` |
pub(crate) fn enumeration_unspecified<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Defaulted<'de>,
{
    T::read(deserializer, "unspecified")
}
