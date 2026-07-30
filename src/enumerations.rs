//! The code lists SIRI defines as XML Schema enumerations.
//!
//! Each type here transcribes one `xsd:simpleType` restriction: the variants are
//! the schema's tokens, in schema order, and each carries the schema's own
//! description. `XSD_TYPE` names the type transcribed and `ALL` lists every value,
//! which is what the conformance tests check the transcription against.
//!
//! Several of these are open code lists that mix SIRI's own vocabulary with TPEG
//! and DATEX II identifiers; those identifiers keep their published spelling even
//! where it is not idiomatic Rust.

siri_enum! {
    /// Delivery Method: Fetched or Direct Delivery.
    DeliveryMethod as "DeliveryMethodEnumeration" {
        /// direct
        Direct = "direct",
        /// fetched
        Fetched = "fetched",
    }
}

siri_enum! {
    /// Allowed values for predictors.
    Predictors as "PredictorsEnumeration" {
        /// avmsOnly
        AvmsOnly = "avmsOnly",
        /// anyone
        Anyone = "anyone",
    }
}

siri_enum! {
    /// Enumeration of communications transport method usage.
    CommunicationsTransportMethod as "CommunicationsTransportMethodEnumeration" {
        /// httpPost
        HttpPost = "httpPost",
        /// other
        Other = "other",
        /// wsdlSoap
        WsdlSoap = "wsdlSoap",
        /// wsdlSoapDocumentLiteral
        WsdlSoapDocumentLiteral = "wsdlSoapDocumentLiteral",
        /// httpUrlJSON
        HttpUrlJSON = "httpUrlJSON",
        /// httpUrlProtoBuffers
        HttpUrlProtoBuffers = "httpUrlProtoBuffers",
    }
}

siri_enum! {
    /// Enumeration of compression usage.
    CompressionMethod as "CompressionMethodEnumeration" {
        /// gzip
        Gzip = "gzip",
        /// none
        None = "none",
        /// other
        Other = "other",
    }
}

siri_enum! {
    /// Detail Levels for Stop Points Discovery Request. (since SIRI 2.0)
    StopPointsDetail as "StopPointsDetailEnumeration" {
        /// Return only the name and identifier of the stop.
        Minimum = "minimum",
        /// Return name, dientifier and coordinates of the stop.
        Normal = "normal",
        /// Return all available data for each stop.
        Full = "full",
    }
}

siri_enum! {
    /// Detail Levels for Lines Discovery Request. (since SIRI 2.0)
    LinesDetail as "LinesDetailEnumeration" {
        /// Return only the name and identifier of the stop.
        Minimum = "minimum",
        /// Return name, dientifier and coordinates of the stop.
        Normal = "normal",
        /// stops
        Stops = "stops",
        /// Return all available data for each stop.
        Full = "full",
    }
}

siri_enum! {
    /// Values for ModesOfTransport : TPEG Pti01 and Pts001 "ModeOfTransport".
    #[allow(non_camel_case_types)]
    VehicleModesOfTransport as "VehicleModesOfTransportEnumeration" {
        /// air
        Air = "air",
        /// bus
        Bus = "bus",
        /// coach
        Coach = "coach",
        /// (SIRI 2.1)
        Ferry = "ferry",
        /// metro
        Metro = "metro",
        /// rail
        Rail = "rail",
        /// trolleyBus
        TrolleyBus = "trolleyBus",
        /// tram
        Tram = "tram",
        /// water
        Water = "water",
        /// (SIRI 2.1)
        Cableway = "cableway",
        /// funicular
        Funicular = "funicular",
        /// lift
        Lift = "lift",
        /// (SIRI 2.1)
        SnowAndIce = "snowAndIce",
        /// Placeholder value if mode of transport is different from all other enumerations in this list (SIRI 2.1) - same meaning as 'undefinedModeOfTransport'.
        Other = "other",
        /// TPEG Pts1_0 - mode of transport is not known to the source system.
        Unknown = "unknown",
        /// TPEG Pts1_1 - use 'air' instead.
        AirService = "airService",
        /// TPEG Pts1_2 (SIRI 2.1) - see also 'cableway'.
        GondolaCableCarService = "gondolaCableCarService",
        /// TPEG Pts1_3 (SIRI 2.1)
        ChairliftService = "chairliftService",
        /// TPEG Pts1_4 (SIRI 2.1) - use 'lift' instead.
        ElevatorService = "elevatorService",
        /// TPEG Pts1_5 - use 'rail' instead.
        RailwayService = "railwayService",
        /// TPEG Pts1_6 - see also 'urbanRail'.
        UrbanRailwayService = "urbanRailwayService",
        /// TPEG Pts1_7 (SIRI 2.1)
        LightRailwayService = "lightRailwayService",
        /// TPEG Pts1_8 (SIRI 2.1)
        RackRailService = "rackRailService",
        /// TPEG Pts1_9 - use 'funicular' instead.
        FunicularService = "funicularService",
        /// TPEG Pts1_10 - use 'bus' instead.
        BusService = "busService",
        /// TPEG Pts1_11 (SIRI 2.1) - use 'trolleyBus' instead.
        TrolleybusService = "trolleybusService",
        /// TPEG Pts1_12 - use 'coach' instead.
        CoachService = "coachService",
        /// TPEG Pts1_13 - use 'taxi' instead.
        TaxiService = "taxiService",
        /// TPEG Pts1_14 (SIRI 2.1)
        RentalVehicle = "rentalVehicle",
        /// TPEG Pts1_15 - use 'water' instead.
        WaterTransportService = "waterTransportService",
        /// TPEG Pts1_16 (SIRI 2.1)
        CableDrawnBoatService = "cableDrawnBoatService",
        /// TPEG Pts1_255 (SIRI 2.1) - mode of transport is not supported in this list.
        UndefinedModeOfTransport = "undefinedModeOfTransport",
        /// suburbanRail
        SuburbanRail = "suburbanRail",
        /// See also 'suburbanRail'.
        SuburbanRailwayService = "suburbanRailwayService",
        /// urbanRail
        UrbanRail = "urbanRail",
        /// underground
        Underground = "underground",
        /// See also 'underground'.
        UndergroundService = "undergroundService",
        /// Use 'metro' instead.
        MetroService = "metroService",
        /// Use 'trolleyBus' instead.
        TrolleyBusService = "trolleyBusService",
        /// Use 'tram' instead.
        TramService = "tramService",
        /// Use 'water' instead.
        WaterTransport = "waterTransport",
        /// Use 'ferry' instead.
        FerryService = "ferryService",
        /// See also 'cableway'.
        Telecabin = "telecabin",
        /// See also 'telecabin'.
        TelecabinService = "telecabinService",
        /// taxi
        Taxi = "taxi",
        /// selfDrive
        SelfDrive = "selfDrive",
        /// all
        All = "all",
        /// See also 'all'.
        AllServices = "allServices",
        /// allServicesExcept
        AllServicesExcept = "allServicesExcept",
        /// DEPRECATED since SIRI 2.1
        Pti1_0 = "pti1_0",
        /// DEPRECATED since SIRI 2.1
        Pti1_1 = "pti1_1",
        /// DEPRECATED since SIRI 2.1
        Pti1_2 = "pti1_2",
        /// DEPRECATED since SIRI 2.1
        Pti1_3 = "pti1_3",
        /// DEPRECATED since SIRI 2.1
        Pti1_4 = "pti1_4",
        /// DEPRECATED since SIRI 2.1
        Pti1_5 = "pti1_5",
        /// DEPRECATED since SIRI 2.1
        Pti1_6 = "pti1_6",
        /// DEPRECATED since SIRI 2.1
        Pti1_7 = "pti1_7",
        /// DEPRECATED since SIRI 2.1
        Pti1_8 = "pti1_8",
        /// DEPRECATED since SIRI 2.1
        Pti1_9 = "pti1_9",
        /// DEPRECATED since SIRI 2.1
        Pti1_10 = "pti1_10",
        /// DEPRECATED since SIRI 2.1
        Pti1_11 = "pti1_11",
        /// DEPRECATED since SIRI 2.1
        Pti1_12 = "pti1_12",
        /// DEPRECATED since SIRI 2.1
        Pti1_13 = "pti1_13",
        /// DEPRECATED since SIRI 2.1
        Pti1_14 = "pti1_14",
        /// DEPRECATED since SIRI 2.1
        Pti1_15 = "pti1_15",
        /// DEPRECATED since SIRI 2.1
        Pti1_16 = "pti1_16",
        /// DEPRECATED since SIRI 2.1
        Pti1_17 = "pti1_17",
        /// DEPRECATED since SIRI 2.1
        Pti1_18 = "pti1_18",
    }
}

siri_enum! {
    /// Allowed categroies of access to STOP PLACE.
    AccessModes as "AccessModesEnumeration" {
        /// foot
        Foot = "foot",
        /// bicycle
        Bicycle = "bicycle",
        /// car
        Car = "car",
        /// taxi
        Taxi = "taxi",
        /// shuttle
        Shuttle = "shuttle",
    }
}

siri_enum! {
    /// Values for Air ModesOfTransport: TPEG pti_table_08 and pts_table_108.
    #[allow(non_camel_case_types)]
    AirSubmodesOfTransport as "AirSubmodesOfTransportEnumeration" {
        /// TPEG Pts108_0 - submode of transport is not known to the source system.
        Unknown = "unknown",
        /// (SIRI 2.1) - see also 'undefinedAircraftService'.
        Undefined = "undefined",
        /// internationalFlight
        InternationalFlight = "internationalFlight",
        /// domesticFlight
        DomesticFlight = "domesticFlight",
        /// intercontinentalFlight
        IntercontinentalFlight = "intercontinentalFlight",
        /// domesticScheduledFlight
        DomesticScheduledFlight = "domesticScheduledFlight",
        /// shuttleFlight
        ShuttleFlight = "shuttleFlight",
        /// intercontinentalCharterFlight
        IntercontinentalCharterFlight = "intercontinentalCharterFlight",
        /// internationalCharterFlight
        InternationalCharterFlight = "internationalCharterFlight",
        /// roundTripCharterFlight
        RoundTripCharterFlight = "roundTripCharterFlight",
        /// sightseeingFlight
        SightseeingFlight = "sightseeingFlight",
        /// helicopterService
        HelicopterService = "helicopterService",
        /// domesticCharterFlight
        DomesticCharterFlight = "domesticCharterFlight",
        /// SchengenAreaFlight
        SchengenAreaFlight = "SchengenAreaFlight",
        /// TPEG Pts108_13
        AirshipService = "airshipService",
        /// shortHaulInternationalFlight
        ShortHaulInternationalFlight = "shortHaulInternationalFlight",
        /// TPEG Pts108_1 (SIRI 2.1) - see also 'internationalFlight'.
        InternationalAirService = "internationalAirService",
        /// TPEG Pts108_2 (SIRI 2.1) - see also 'domesticFlight'.
        NationalAirService = "nationalAirService",
        /// TPEG Pts108_3 (SIRI 2.1) - see also 'intercontinentalFlight'.
        IntercontinentalAirService = "intercontinentalAirService",
        /// TPEG Pts108_4 (SIRI 2.1) - see also 'domesticScheduledFlight'.
        NationalScheduledAirService = "nationalScheduledAirService",
        /// TPEG Pts108_5 (SIRI 2.1) - see also 'shuttleFlight'.
        ShuttleAirService = "shuttleAirService",
        /// TPEG Pts108_6 (SIRI 2.1) - see also 'intercontinentalCharterFlight'.
        IntercontinentalAirCharterService = "intercontinentalAirCharterService",
        /// TPEG Pts108_7 (SIRI 2.1) - see also 'intercontinentalCharterFlight'.
        InternationalAirCharterService = "internationalAirCharterService",
        /// TPEG Pts108_8 (SIRI 2.1) - see also 'roundTripCharterFlight'.
        RoundTripAirCharterService = "roundTripAirCharterService",
        /// TPEG Pts108_9 (SIRI 2.1) - see also 'sightseeingFlight'.
        SightseeingAirService = "sightseeingAirService",
        /// TPEG Pts108_10 (SIRI 2.1) - see also 'helicopterService'.
        HelicopterAirService = "helicopterAirService",
        /// TPEG Pts108_11 (SIRI 2.1) - see also 'domesticCharterFlight'.
        DomesticAirCharterService = "domesticAirCharterService",
        /// TPEG Pts108_12 (SIRI 2.1) - see also 'SchengenAreaFlight'.
        SchengenAreaAirService = "SchengenAreaAirService",
        /// TPEG Pts108_14 (SIRI 2.1)
        OnDemandService = "onDemandService",
        /// TPEG Pts108_15 (SIRI 2.1) - see also 'undefinedAircraftService'.
        UndefinedAirService = "undefinedAirService",
        /// Submode of transport is not supported in this list.
        UndefinedAircraftService = "undefinedAircraftService",
        /// allAirServices
        AllAirServices = "allAirServices",
        /// DEPRECATED since SIRI 2.1
        Pti8_0 = "pti8_0",
        /// DEPRECATED since SIRI 2.1
        Pti8_1 = "pti8_1",
        /// DEPRECATED since SIRI 2.1
        Pti8_2 = "pti8_2",
        /// DEPRECATED since SIRI 2.1
        Pti8_3 = "pti8_3",
        /// DEPRECATED since SIRI 2.1
        Pti8_4 = "pti8_4",
        /// DEPRECATED since SIRI 2.1
        Pti8_5 = "pti8_5",
        /// DEPRECATED since SIRI 2.1
        Pti8_6 = "pti8_6",
        /// DEPRECATED since SIRI 2.1
        Pti8_7 = "pti8_7",
        /// DEPRECATED since SIRI 2.1
        Pti8_8 = "pti8_8",
        /// DEPRECATED since SIRI 2.1
        Pti8_9 = "pti8_9",
        /// DEPRECATED since SIRI 2.1
        Pti8_10 = "pti8_10",
        /// DEPRECATED since SIRI 2.1
        Pti8_11 = "pti8_11",
        /// DEPRECATED since SIRI 2.1
        Pti8_12 = "pti8_12",
        /// DEPRECATED since SIRI 2.1
        Pti8_13 = "pti8_13",
        /// DEPRECATED since SIRI 2.1
        Pti8_14 = "pti8_14",
        /// DEPRECATED since SIRI 2.1
        Pti8_255 = "pti8_255",
        /// DEPRECATED since SIRI 2.1
        Loc15_0 = "loc15_0",
        /// DEPRECATED since SIRI 2.1
        Loc15_1 = "loc15_1",
        /// DEPRECATED since SIRI 2.1
        Loc15_2 = "loc15_2",
        /// DEPRECATED since SIRI 2.1
        Loc14_3 = "loc14_3",
        /// DEPRECATED since SIRI 2.1
        Loc15_4 = "loc15_4",
        /// DEPRECATED since SIRI 2.1
        Loc15_5 = "loc15_5",
        /// DEPRECATED since SIRI 2.1
        Loc15_6 = "loc15_6",
        /// DEPRECATED since SIRI 2.1
        Loc15_7 = "loc15_7",
        /// DEPRECATED since SIRI 2.1
        Loc15_8 = "loc15_8",
        /// DEPRECATED since SIRI 2.1
        Loc15_9 = "loc15_9",
        /// DEPRECATED since SIRI 2.1
        Loc15_10 = "loc15_10",
        /// DEPRECATED since SIRI 2.1
        Loc15_255 = "loc15_255",
    }
}

siri_enum! {
    /// Values for Bus ModesOfTransport: TPEG pti_table_05, pts_table_105 and loc_table_10.
    #[allow(non_camel_case_types)]
    BusSubmodesOfTransport as "BusSubmodesOfTransportEnumeration" {
        /// TPEG Pts105_0 - submode of transport is not known to the source system.
        Unknown = "unknown",
        /// Submode of transport is not supported in this list.
        Undefined = "undefined",
        /// (SIRI 2.1) - see also 'localBusService'.
        LocalBus = "localBus",
        /// regionalBus
        RegionalBus = "regionalBus",
        /// expressBus
        ExpressBus = "expressBus",
        /// nightBus
        NightBus = "nightBus",
        /// postBus
        PostBus = "postBus",
        /// specialNeedsBus
        SpecialNeedsBus = "specialNeedsBus",
        /// mobilityBus
        MobilityBus = "mobilityBus",
        /// mobilityBusForRegisteredDisabled
        MobilityBusForRegisteredDisabled = "mobilityBusForRegisteredDisabled",
        /// sightseeingBus
        SightseeingBus = "sightseeingBus",
        /// shuttleBus
        ShuttleBus = "shuttleBus",
        /// (SIRI 2.1)
        HighFrequencyBus = "highFrequencyBus",
        /// (SIRI 2.1)
        DedicatedLaneBus = "dedicatedLaneBus",
        /// schoolBus
        SchoolBus = "schoolBus",
        /// schoolAndPublicServiceBus
        SchoolAndPublicServiceBus = "schoolAndPublicServiceBus",
        /// railReplacementBus
        RailReplacementBus = "railReplacementBus",
        /// demandAndResponseBus
        DemandAndResponseBus = "demandAndResponseBus",
        /// airportLinkBus
        AirportLinkBus = "airportLinkBus",
        /// TPEG Pts105_1 (SIRI 2.1) - see also 'regionalBus'.
        RegionalBusService = "regionalBusService",
        /// TPEG Pts105_2 (SIRI 2.1)
        AdditionalBusService = "additionalBusService",
        /// TPEG Pts105_3 (SIRI 2.1) - see also 'expressBus'.
        ExpressBusService = "expressBusService",
        /// TPEG Pts105_4 (SIRI 2.1)
        StoppingBusService = "stoppingBusService",
        /// TPEG Pts105_5 (SIRI 2.1)
        LocalBusService = "localBusService",
        /// TPEG Pts105_6 (SIRI 2.1) - see also 'nightBus'.
        NightBusService = "nightBusService",
        /// TPEG Pts105_7 (SIRI 2.1) - see also 'postBus'.
        PostBusService = "postBusService",
        /// TPEG Pts105_8 (SIRI 2.1) - see also 'specialNeedsBus'.
        SpecialNeedsBusService = "specialNeedsBusService",
        /// TPEG Pts105_9 (SIRI 2.1) - see also 'mobilityBus'.
        MobilityBusService = "mobilityBusService",
        /// TPEG Pts105_10 (SIRI 2.1) - see also 'mobilityBusForRegisteredDisabled'.
        MobilityBusForRegisteredDisabledService = "mobilityBusForRegisteredDisabledService",
        /// TPEG Pts105_11 (SIRI 2.1) - see also 'sightseeingBus'.
        SightseeingBusService = "sightseeingBusService",
        /// TPEG Pts105_12 (SIRI 2.1) - see also 'shuttleBus'.
        ShuttleBusService = "shuttleBusService",
        /// TPEG Pts105_13 (SIRI 2.1) - see also 'schoolBus'.
        SchoolBusService = "schoolBusService",
        /// TPEG Pts105_14 (SIRI 2.1) - see also 'schoolAndPublicServiceBus'.
        SchoolAndPublicServiceBusService = "schoolAndPublicServiceBusService",
        /// TPEG Pts105_15 (SIRI 2.1) - see also 'railReplacementBus'.
        RailReplacementBusService = "railReplacementBusService",
        /// TPEG Pts105_16 (SIRI 2.1) - see also 'demandAndResponseBus'.
        DemandAndResponseBusService = "demandAndResponseBusService",
        /// TPEG Pts105_255 (SIRI 2.1) - see also 'undefined'.
        UndefinedBusService = "undefinedBusService",
        /// bus
        Bus = "bus",
        /// allBusServices
        AllBusServices = "allBusServices",
        /// DEPRECATED since SIRI 2.1
        Pti5_0 = "pti5_0",
        /// DEPRECATED since SIRI 2.1
        Pti5_1 = "pti5_1",
        /// DEPRECATED since SIRI 2.1
        Pti5_2 = "pti5_2",
        /// DEPRECATED since SIRI 2.1
        Pti5_3 = "pti5_3",
        /// DEPRECATED since SIRI 2.1
        Pti5_4 = "pti5_4",
        /// DEPRECATED since SIRI 2.1
        Pti5_5 = "pti5_5",
        /// DEPRECATED since SIRI 2.1
        Pti5_6 = "pti5_6",
        /// DEPRECATED since SIRI 2.1
        Pti5_7 = "pti5_7",
        /// DEPRECATED since SIRI 2.1
        Pti5_8 = "pti5_8",
        /// DEPRECATED since SIRI 2.1
        Pti5_9 = "pti5_9",
        /// DEPRECATED since SIRI 2.1
        Pti5_10 = "pti5_10",
        /// DEPRECATED since SIRI 2.1
        Pti5_11 = "pti5_11",
        /// DEPRECATED since SIRI 2.1
        Pti5_12 = "pti5_12",
        /// DEPRECATED since SIRI 2.1
        Pti5_13 = "pti5_13",
        /// DEPRECATED since SIRI 2.1
        Pti5_14 = "pti5_14",
        /// DEPRECATED since SIRI 2.1
        Pti5_15 = "pti5_15",
        /// DEPRECATED since SIRI 2.1
        Pti5_16 = "pti5_16",
        /// DEPRECATED since SIRI 2.1
        Pti5_255 = "pti5_255",
        /// DEPRECATED since SIRI 2.1
        Loc_10 = "loc_10",
        /// DEPRECATED since SIRI 2.1
        Loc10_0 = "loc10_0",
        /// DEPRECATED since SIRI 2.1
        Loc10_1 = "loc10_1",
        /// DEPRECATED since SIRI 2.1
        Loc10_2 = "loc10_2",
        /// DEPRECATED since SIRI 2.1
        Loc10_4 = "loc10_4",
        /// DEPRECATED since SIRI 2.1
        Loc10_5 = "loc10_5",
        /// DEPRECATED since SIRI 2.1
        Loc10_6 = "loc10_6",
        /// DEPRECATED since SIRI 2.1
        Loc10_7 = "loc10_7",
        /// DEPRECATED since SIRI 2.1
        Loc10_8 = "loc10_8",
        /// DEPRECATED since SIRI 2.1
        Loc10_9 = "loc10_9",
        /// DEPRECATED since SIRI 2.1
        Loc10_13 = "loc10_13",
        /// DEPRECATED since SIRI 2.1
        Loc10_255 = "loc10_255",
    }
}

siri_enum! {
    /// Values for Coach ModesOfTransport: TPEG pti_table_03 and pts_table_103.
    #[allow(non_camel_case_types)]
    CoachSubmodesOfTransport as "CoachSubmodesOfTransportEnumeration" {
        /// TPEG Pts103_0 - submode of transport is not known to the source system.
        Unknown = "unknown",
        /// Submode of transport is not supported in this list.
        Undefined = "undefined",
        /// (SIRI 2.1) - see also 'internationalCoachService'.
        InternationalCoach = "internationalCoach",
        /// (SIRI 2.1) - see also 'nationalCoachService'.
        NationalCoach = "nationalCoach",
        /// (SIRI 2.1) - see also 'shuttleCoachService'.
        ShuttleCoach = "shuttleCoach",
        /// (SIRI 2.1) - see also 'regionalCoachService'.
        RegionalCoach = "regionalCoach",
        /// (SIRI 2.1) - see also 'specialCoachService'.
        SpecialCoach = "specialCoach",
        /// (SIRI 2.1)
        SchoolCoach = "schoolCoach",
        /// (SIRI 2.1) - see also 'sightseeingCoachService'.
        SightseeingCoach = "sightseeingCoach",
        /// (SIRI 2.1) - see also 'touristCoachService'.
        TouristCoach = "touristCoach",
        /// (SIRI 2.1) - see also 'commuterCoachService'.
        CommuterCoach = "commuterCoach",
        /// TPEG Pts103_1
        InternationalCoachService = "internationalCoachService",
        /// TPEG Pts103_2
        NationalCoachService = "nationalCoachService",
        /// TPEG Pts103_3
        ShuttleCoachService = "shuttleCoachService",
        /// TPEG Pts103_4
        RegionalCoachService = "regionalCoachService",
        /// TPEG Pts103_5 (SIRI 2.1)
        AdditionalCoachService = "additionalCoachService",
        /// TPEG Pts103_6 (SIRI 2.1)
        NightCoachService = "nightCoachService",
        /// TPEG Pts103_7
        SpecialCoachService = "specialCoachService",
        /// TPEG Pts103_8
        SightseeingCoachService = "sightseeingCoachService",
        /// TPEG Pts103_9
        TouristCoachService = "touristCoachService",
        /// TPEG Pts103_10
        CommuterCoachService = "commuterCoachService",
        /// TPEG Pts103_11 (SIRI 2.1)
        OnDemandService = "onDemandService",
        /// TPEG Pts103_255 (SIRI 2.1) - see also 'undefined'.
        UndefinedCoachService = "undefinedCoachService",
        /// allCoachServices
        AllCoachServices = "allCoachServices",
        /// DEPRECATED since SIRI 2.1
        Pti3_0 = "pti3_0",
        /// DEPRECATED since SIRI 2.1
        Pti3_1 = "pti3_1",
        /// DEPRECATED since SIRI 2.1
        Pti3_2 = "pti3_2",
        /// DEPRECATED since SIRI 2.1
        Pti3_3 = "pti3_3",
        /// DEPRECATED since SIRI 2.1
        Pti3_4 = "pti3_4",
        /// DEPRECATED since SIRI 2.1
        Pti3_5 = "pti3_5",
        /// DEPRECATED since SIRI 2.1
        Pti3_6 = "pti3_6",
        /// DEPRECATED since SIRI 2.1
        Pti3_7 = "pti3_7",
        /// DEPRECATED since SIRI 2.1
        Pti3_8 = "pti3_8",
        /// DEPRECATED since SIRI 2.1
        Pti3_9 = "pti3_9",
        /// DEPRECATED since SIRI 2.1
        Pti3_255 = "pti3_255",
    }
}

siri_enum! {
    /// Values for Metro ModesOfTransport: TPEG pti_table_04 and pts_table_104.
    #[allow(non_camel_case_types)]
    MetroSubmodesOfTransport as "MetroSubmodesOfTransportEnumeration" {
        /// TPEG Pts104_0 - submode of transport is not known to the source system.
        Unknown = "unknown",
        /// Submode of transport is not supported in this list.
        Undefined = "undefined",
        /// metro
        Metro = "metro",
        /// tube
        Tube = "tube",
        /// urbanRailway
        UrbanRailway = "urbanRailway",
        /// allRailServices
        AllRailServices = "allRailServices",
        /// TPEG Pts104_1 (SIRI 2.1) - see also 'metro'.
        MetroService = "metroService",
        /// TPEG Pts104_2 (SIRI 2.1)
        NightMetroService = "nightMetroService",
        /// TPEG Pts104_3 (SIRI 2.1)
        ExpressMetroService = "expressMetroService",
        /// TPEG Pts104_255 (SIRI 2.1) - see also 'undefined'.
        UndefinedUrbanRailwayService = "undefinedUrbanRailwayService",
        /// DEPRECATED since SIRI 2.1
        Pti4_0 = "pti4_0",
        /// DEPRECATED since SIRI 2.1
        Pti4_1 = "pti4_1",
        /// DEPRECATED since SIRI 2.1
        Pti4_2 = "pti4_2",
        /// DEPRECATED since SIRI 2.1
        Pti4_3 = "pti4_3",
        /// DEPRECATED since SIRI 2.1
        Pti4_4 = "pti4_4",
        /// DEPRECATED since SIRI 2.1
        Pti4_255 = "pti4_255",
    }
}

siri_enum! {
    /// Values for Rail ModesOfTransport: TPEG pti_table_02, pts_table_102 "RailwayService" and train link loc_table_13.
    #[allow(non_camel_case_types)]
    RailSubmodesOfTransport as "RailSubmodesOfTransportEnumeration" {
        /// TPEG Pts102_0 - submode of transport is not known to the source system.
        Unknown = "unknown",
        /// local
        Local = "local",
        /// (SIRI 2.1)
        HighSpeedRail = "highSpeedRail",
        /// suburbanRailway
        SuburbanRailway = "suburbanRailway",
        /// regionalRail
        RegionalRail = "regionalRail",
        /// (SIRI 2.1)
        InterregionalRail = "interregionalRail",
        /// (SIRI 2.1)
        LongDistance = "longDistance",
        /// international
        International = "international",
        /// TPEG Pts105_6
        SleeperRailService = "sleeperRailService",
        /// (SIRI 2.1)
        NightRail = "nightRail",
        /// carTransportRailService
        CarTransportRailService = "carTransportRailService",
        /// touristRailway
        TouristRailway = "touristRailway",
        /// (SIRI 2.1)
        AirportLinkRail = "airportLinkRail",
        /// railShuttle
        RailShuttle = "railShuttle",
        /// TPEG Pts105_13
        ReplacementRailService = "replacementRailService",
        /// (SIRI 2.1)
        SpecialTrain = "specialTrain",
        /// (SIRI 2.1)
        CrossCountryRail = "crossCountryRail",
        /// rackAndPinionRailway
        RackAndPinionRailway = "rackAndPinionRailway",
        /// TPEG Pts102_1 - see also 'highSpeedRail'.
        HighSpeedRailService = "highSpeedRailService",
        /// TPEG Pts102_2 (SIRI 2.1) - see also 'international'.
        LongDistanceInternationalRailService = "longDistanceInternationalRailService",
        /// TPEG Pts102_3 (SIRI 2.1) - see also 'longDistance'.
        LongDistanceRailService = "longDistanceRailService",
        /// TPEG Pts102_4 (SIRI 2.1)
        InterRegionalExpressRailService = "interRegionalExpressRailService",
        /// TPEG Pts105_5 - see also 'interregionalRail'.
        InterRegionalRailService = "interRegionalRailService",
        /// TPEG Pts105_7 (SIRI 2.1)
        RegionalExpressRailService = "regionalExpressRailService",
        /// TPEG Pts105_8 (SIRI 2.1) - see also 'regionalRail'.
        RegionalRailService = "regionalRailService",
        /// TPEG Pts105_9 (SIRI 2.1) - see also 'touristRailway'.
        TouristRailwayService = "touristRailwayService",
        /// TPEG Pts105_10 (SIRI 2.1) - see also 'railShuttle'.
        RailShuttleService = "railShuttleService",
        /// TPEG Pts105_11 (SIRI 2.1)
        SuburbanRailService = "suburbanRailService",
        /// TPEG Pts105_12 (SIRI 2.1) - see also 'nightRail'.
        SuburbanNightRailService = "suburbanNightRailService",
        /// TPEG Pts105_14 (SIRI 2.1) - see also 'specialTrain'.
        SpecialRailService = "specialRailService",
        /// TPEG Pts105_15
        LorryTransportRailService = "lorryTransportRailService",
        /// TPEG Pts105_17 (SIRI 2.1) - see also 'vehicleRailTransportService'.
        VehicleTransportRailService = "vehicleTransportRailService",
        /// TPEG Pts105_18 (SIRI 2.1)
        VehicleTunnelTransportRailService = "vehicleTunnelTransportRailService",
        /// TPEG Pts105_19 (SIRI 2.1) - see also 'additionalTrainService'.
        AdditionalRailService = "additionalRailService",
        /// TPEG Pts105_255 (SIRI 2.1) - see also 'undefined'.
        UndefinedRailService = "undefinedRailService",
        /// See also 'longDistance'.
        LongDistanceTrain = "longDistanceTrain",
        /// specialTrainService
        SpecialTrainService = "specialTrainService",
        /// crossCountryRailService
        CrossCountryRailService = "crossCountryRailService",
        /// vehicleRailTransportService
        VehicleRailTransportService = "vehicleRailTransportService",
        /// additionalTrainService
        AdditionalTrainService = "additionalTrainService",
        /// allRailServices
        AllRailServices = "allRailServices",
        /// Submode of transport is not supported in this list.
        Undefined = "undefined",
        /// DEPRECATED since SIRI 2.1
        Interbational = "interbational",
        /// DEPRECATED since SIRI 2.1
        Pti2_0 = "pti2_0",
        /// DEPRECATED since SIRI 2.1
        Pti2_1 = "pti2_1",
        /// DEPRECATED since SIRI 2.1
        Pti2_2 = "pti2_2",
        /// DEPRECATED since SIRI 2.1
        Pti2_3 = "pti2_3",
        /// DEPRECATED since SIRI 2.1
        Pti2_4 = "pti2_4",
        /// DEPRECATED since SIRI 2.1
        Pti2_5 = "pti2_5",
        /// DEPRECATED since SIRI 2.1
        Pti2_6 = "pti2_6",
        /// DEPRECATED since SIRI 2.1
        Pti2_7 = "pti2_7",
        /// DEPRECATED since SIRI 2.1
        Pti2_8 = "pti2_8",
        /// DEPRECATED since SIRI 2.1
        Pti2_9 = "pti2_9",
        /// DEPRECATED since SIRI 2.1
        Pti2_10 = "pti2_10",
        /// DEPRECATED since SIRI 2.1
        Pti2_11 = "pti2_11",
        /// DEPRECATED since SIRI 2.1
        Pti2_12 = "pti2_12",
        /// DEPRECATED since SIRI 2.1
        Pti2_13 = "pti2_13",
        /// DEPRECATED since SIRI 2.1
        Pti2_14 = "pti2_14",
        /// DEPRECATED since SIRI 2.1
        Pti2_15 = "pti2_15",
        /// DEPRECATED since SIRI 2.1
        Pti2_16 = "pti2_16",
        /// DEPRECATED since SIRI 2.1
        Pti2_17 = "pti2_17",
        /// DEPRECATED since SIRI 2.1
        Pti2_255 = "pti2_255",
        /// DEPRECATED since SIRI 2.1
        Loc13_0 = "loc13_0",
        /// DEPRECATED since SIRI 2.1
        Loc13_1 = "loc13_1",
        /// DEPRECATED since SIRI 2.1
        Loc13_2 = "loc13_2",
        /// DEPRECATED since SIRI 2.1
        Loc13_3 = "loc13_3",
        /// DEPRECATED since SIRI 2.1
        Loc13_4 = "loc13_4",
        /// DEPRECATED since SIRI 2.1
        Loc13_5 = "loc13_5",
        /// DEPRECATED since SIRI 2.1
        Loc13_6 = "loc13_6",
        /// DEPRECATED since SIRI 2.1
        Loc13_7 = "loc13_7",
        /// DEPRECATED since SIRI 2.1
        Loc13_8 = "loc13_8",
    }
}

siri_enum! {
    /// Values for Tram ModesOfTransport: TPEG pti_table_06, pts_table_104 and loc_table_12.
    #[allow(non_camel_case_types)]
    TramSubmodesOfTransport as "TramSubmodesOfTransportEnumeration" {
        /// TPEG Pts104_0 - submode of transport is not known to the source system.
        Unknown = "unknown",
        /// (SIRI 2.1) - see also 'undefinedTramService'.
        Undefined = "undefined",
        /// cityTram
        CityTram = "cityTram",
        /// (SIRI 2.1) - see also 'localTramService'.
        LocalTram = "localTram",
        /// regionalTram
        RegionalTram = "regionalTram",
        /// sightseeingTram
        SightseeingTram = "sightseeingTram",
        /// shuttleTram
        ShuttleTram = "shuttleTram",
        /// (SIRI 2.1)
        TrainTram = "trainTram",
        /// TPEG Pts104_4 (SIRI 2.1)
        TramService = "tramService",
        /// TPEG Pts104_5 (SIRI 2.1) - see also 'cityTram'.
        CityTramService = "cityTramService",
        /// TPEG Pts104_6 (SIRI 2.1) - see also 'regionalTram'.
        RegionalTramService = "regionalTramService",
        /// TPEG Pts104_7 (SIRI 2.1) - see also 'sightseeingTram'.
        SightseeingTramService = "sightseeingTramService",
        /// TPEG Pts104_8 (SIRI 2.1)
        NightTramService = "nightTramService",
        /// TPEG Pts104_9 (SIRI 2.1) - see also 'shuttleTram'.
        ShuttleTramService = "shuttleTramService",
        /// TPEG Pts104_255 (SIRI 2.1) - see also 'undefined'.
        UndefinedUrbanRailwayService = "undefinedUrbanRailwayService",
        /// localTramService
        LocalTramService = "localTramService",
        /// Submode of transport is not supported in this list.
        UndefinedTramService = "undefinedTramService",
        /// allTramServices
        AllTramServices = "allTramServices",
        /// DEPRECATED since SIRI 2.1
        Pti6_0 = "pti6_0",
        /// DEPRECATED since SIRI 2.1
        Pti6_1 = "pti6_1",
        /// DEPRECATED since SIRI 2.1
        Pti6_2 = "pti6_2",
        /// DEPRECATED since SIRI 2.1
        Pti6_3 = "pti6_3",
        /// DEPRECATED since SIRI 2.1
        Pti6_4 = "pti6_4",
        /// DEPRECATED since SIRI 2.1
        Pti6_5 = "pti6_5",
        /// DEPRECATED since SIRI 2.1
        Pti6_6 = "pti6_6",
        /// DEPRECATED since SIRI 2.1
        Pti6_255 = "pti6_255",
        /// DEPRECATED since SIRI 2.1
        Loc12_0 = "loc12_0",
        /// DEPRECATED since SIRI 2.1
        Loc12_1 = "loc12_1",
        /// DEPRECATED since SIRI 2.1
        Loc12_2 = "loc12_2",
        /// DEPRECATED since SIRI 2.1
        Loc12_255 = "loc12_255",
    }
}

siri_enum! {
    /// Values for Water ModesOfTransport: TPEG pti_table_07 and pts_table_107.
    #[allow(non_camel_case_types)]
    WaterSubmodesOfTransport as "WaterSubmodesOfTransportEnumeration" {
        /// TPEG Pts107_0 - submode of transport is not known to the source system.
        Unknown = "unknown",
        /// (SIRI 2.1) - see also 'undefinedWaterTransport'.
        Undefined = "undefined",
        /// (SIRI 2.1) - see also 'internationalCarFerryService'.
        InternationalCarFerry = "internationalCarFerry",
        /// (SIRI 2.1) - see also 'nationalCarFerryService'.
        NationalCarFerry = "nationalCarFerry",
        /// (SIRI 2.1) - see also 'regionalCarFerryService'.
        RegionalCarFerry = "regionalCarFerry",
        /// (SIRI 2.1) - see also 'localCarFerryService'.
        LocalCarFerry = "localCarFerry",
        /// internationalPassengerFerry
        InternationalPassengerFerry = "internationalPassengerFerry",
        /// nationalPassengerFerry
        NationalPassengerFerry = "nationalPassengerFerry",
        /// regionalPassengerFerry
        RegionalPassengerFerry = "regionalPassengerFerry",
        /// localPassengerFerry
        LocalPassengerFerry = "localPassengerFerry",
        /// postBoat
        PostBoat = "postBoat",
        /// trainFerry
        TrainFerry = "trainFerry",
        /// roadFerryLink
        RoadFerryLink = "roadFerryLink",
        /// airportBoatLink
        AirportBoatLink = "airportBoatLink",
        /// highSpeedVehicleService
        HighSpeedVehicleService = "highSpeedVehicleService",
        /// highSpeedPassengerService
        HighSpeedPassengerService = "highSpeedPassengerService",
        /// sightseeingService
        SightseeingService = "sightseeingService",
        /// schoolBoat
        SchoolBoat = "schoolBoat",
        /// cableFerry
        CableFerry = "cableFerry",
        /// riverBus
        RiverBus = "riverBus",
        /// scheduledFerry
        ScheduledFerry = "scheduledFerry",
        /// shuttleFerryService
        ShuttleFerryService = "shuttleFerryService",
        /// (SIRI 2.1)
        CanalBarge = "canalBarge",
        /// TPEG Pts107_2
        InternationalCarFerryService = "internationalCarFerryService",
        /// TPEG Pts107_3
        NationalCarFerryService = "nationalCarFerryService",
        /// TPEG Pts107_4
        RegionalCarFerryService = "regionalCarFerryService",
        /// TPEG Pts107_5
        LocalCarFerryService = "localCarFerryService",
        /// TPEG Pts107_6 (SIRI 2.1) - see also 'internationalPassengerFerry'.
        InternationalPassengerFerryService = "internationalPassengerFerryService",
        /// TPEG Pts107_7 (SIRI 2.1) - see also 'nationalPassengerFerry'.
        NationalPassengerFerryService = "nationalPassengerFerryService",
        /// TPEG Pts107_8 (SIRI 2.1) - see also 'regionalPassengerFerry'.
        RegionalPassengerFerryService = "regionalPassengerFerryService",
        /// TPEG Pts107_9 (SIRI 2.1) - see also 'localPassengerFerry'.
        LocalPassengerFerryService = "localPassengerFerryService",
        /// TPEG Pts107_10 (SIRI 2.1) - see also 'postBoat'.
        PostBoatService = "postBoatService",
        /// TPEG Pts107_11 (SIRI 2.1) - see also 'trainFerry'.
        TrainFerryService = "trainFerryService",
        /// TPEG Pts107_12 (SIRI 2.1) - see also 'roadFerryLink'.
        RoadLinkFerryService = "roadLinkFerryService",
        /// TPEG Pts107_13 (SIRI 2.1) - see also 'airportBoatLink'.
        AirportLinkFerryService = "airportLinkFerryService",
        /// TPEG Pts107_14 (SIRI 2.1) - see also 'highSpeedVehicleService'.
        CarHighSpeedFerryService = "carHighSpeedFerryService",
        /// TPEG Pts107_15 (SIRI 2.1) - see also 'highSpeedPassengerService'.
        PassengerHighSpeedFerryService = "passengerHighSpeedFerryService",
        /// TPEG Pts107_16 (SIRI 2.1) - see also 'scheduledFerry'.
        ScheduledBoatService = "scheduledBoatService",
        /// TPEG Pts107_17 (SIRI 2.1)
        ScheduledExpressBoatService = "scheduledExpressBoatService",
        /// TPEG Pts107_18 (SIRI 2.1)
        AdditionalBoatService = "additionalBoatService",
        /// TPEG Pts107_19 (SIRI 2.1) - see also 'sightseeingService'.
        SightseeingBoatService = "sightseeingBoatService",
        /// TPEG Pts107_20 (SIRI 2.1) - see also 'schoolBoat'.
        SchoolBoatService = "schoolBoatService",
        /// TPEG Pts107_21 (SIRI 2.1) - see also 'riverBus'.
        RiverBusService = "riverBusService",
        /// TPEG Pts107_22 (SIRI 2.1) - see also 'scheduledFerry'.
        ScheduledFerryService = "scheduledFerryService",
        /// TPEG Pts107_255 (SIRI 2.1) - see also 'undefinedWaterTransport'.
        UndefinedWaterTransportService = "undefinedWaterTransportService",
        /// Submode of transport is not supported in this list.
        UndefinedWaterTransport = "undefinedWaterTransport",
        /// allWaterTransportServices
        AllWaterTransportServices = "allWaterTransportServices",
        /// DEPRECATED since SIRI 2.1
        Pti7_0 = "pti7_0",
        /// DEPRECATED since SIRI 2.1
        Pti7_1 = "pti7_1",
        /// DEPRECATED since SIRI 2.1
        Pti7_2 = "pti7_2",
        /// DEPRECATED since SIRI 2.1
        Pti7_3 = "pti7_3",
        /// DEPRECATED since SIRI 2.1
        Pti7_4 = "pti7_4",
        /// DEPRECATED since SIRI 2.1
        Pti7_5 = "pti7_5",
        /// DEPRECATED since SIRI 2.1
        Pti7_6 = "pti7_6",
        /// DEPRECATED since SIRI 2.1
        Pti7_7 = "pti7_7",
        /// DEPRECATED since SIRI 2.1
        Pti7_8 = "pti7_8",
        /// DEPRECATED since SIRI 2.1
        Pti7_9 = "pti7_9",
        /// DEPRECATED since SIRI 2.1
        Pti7_10 = "pti7_10",
        /// DEPRECATED since SIRI 2.1
        Pti7_11 = "pti7_11",
        /// DEPRECATED since SIRI 2.1
        Pti7_12 = "pti7_12",
        /// DEPRECATED since SIRI 2.1
        Pti7_13 = "pti7_13",
        /// DEPRECATED since SIRI 2.1
        Pti7_14 = "pti7_14",
        /// DEPRECATED since SIRI 2.1
        Pti7_15 = "pti7_15",
        /// DEPRECATED since SIRI 2.1
        Pti7_16 = "pti7_16",
        /// DEPRECATED since SIRI 2.1
        Pti7_17 = "pti7_17",
        /// DEPRECATED since SIRI 2.1
        Pti7_18 = "pti7_18",
        /// DEPRECATED since SIRI 2.1
        Pti7_19 = "pti7_19",
        /// DEPRECATED since SIRI 2.1
        Pti7_20 = "pti7_20",
        /// DEPRECATED since SIRI 2.1
        Pti7_21 = "pti7_21",
        /// DEPRECATED since SIRI 2.1
        Pti7_255 = "pti7_255",
    }
}

siri_enum! {
    /// Values for Telecabin ModesOfTransport: TPEG pti_table_09, pts_table_109 and loc_table_14.
    #[allow(non_camel_case_types)]
    TelecabinSubmodesOfTransport as "TelecabinSubmodesOfTransportEnumeration" {
        /// TPEG Pts109_0 - submode of transport is not known to the source system.
        Unknown = "unknown",
        /// Submode of transport is not supported in this list.
        Undefined = "undefined",
        /// telecabin
        Telecabin = "telecabin",
        /// cableCar
        CableCar = "cableCar",
        /// lift
        Lift = "lift",
        /// chairLift
        ChairLift = "chairLift",
        /// dragLift
        DragLift = "dragLift",
        /// telecabinLink
        TelecabinLink = "telecabinLink",
        /// TPEG Pts109_1 (SIRI 2.1)
        Scheduled = "scheduled",
        /// TPEG Pts109_2 (SIRI 2.1)
        Unscheduled = "unscheduled",
        /// TPEG Pts109_255 (SIRI 2.1) - see also 'undefined'.
        UndefinedTelecabinService = "undefinedTelecabinService",
        /// smallTelecabin
        SmallTelecabin = "smallTelecabin",
        /// eggLift
        EggLift = "eggLift",
        /// mineralBuckets
        MineralBuckets = "mineralBuckets",
        /// allTelecabinServices
        AllTelecabinServices = "allTelecabinServices",
        /// DEPRECATED since SIRI 2.1
        Pti9_0 = "pti9_0",
        /// DEPRECATED since SIRI 2.1
        Pti9_1 = "pti9_1",
        /// DEPRECATED since SIRI 2.1
        Pti9_2 = "pti9_2",
        /// DEPRECATED since SIRI 2.1
        Pti9_3 = "pti9_3",
        /// DEPRECATED since SIRI 2.1
        Pti9_4 = "pti9_4",
        /// DEPRECATED since SIRI 2.1
        Pti9_5 = "pti9_5",
        /// DEPRECATED since SIRI 2.1
        Pti9_6 = "pti9_6",
        /// DEPRECATED since SIRI 2.1
        Pti9_7 = "pti9_7",
        /// DEPRECATED since SIRI 2.1
        Pti9_255 = "pti9_255",
        /// DEPRECATED since SIRI 2.1
        Loc14_0 = "loc14_0",
        /// DEPRECATED since SIRI 2.1
        Loc14_1 = "loc14_1",
        /// DEPRECATED since SIRI 2.1
        Loc14_3 = "loc14_3",
        /// DEPRECATED since SIRI 2.1
        Loc14_4 = "loc14_4",
        /// DEPRECATED since SIRI 2.1
        Loc14_52 = "loc14_52",
        /// DEPRECATED since SIRI 2.1
        Loc14_6 = "loc14_6",
        /// DEPRECATED since SIRI 2.1
        Loc14_7 = "loc14_7",
        /// DEPRECATED since SIRI 2.1
        Loc14_8 = "loc14_8",
        /// DEPRECATED since SIRI 2.1
        Loc14_255 = "loc14_255",
    }
}

siri_enum! {
    /// Values for TPEG Pti26 - Severity
    Severity as "SeverityEnumeration" {
        /// TPEG Pti26_0, unknown
        Unknown = "unknown",
        /// TPEG Pti26_1, very slight
        VerySlight = "verySlight",
        /// TPEG Pti26_2, slight
        Slight = "slight",
        /// TPEG Pti26_3, normal
        Normal = "normal",
        /// TPEG Pti26_4, severe
        Severe = "severe",
        /// TPEG Pti26_5, very severe
        VerySevere = "verySevere",
        /// TPEG Pti26_6, no impact
        NoImpact = "noImpact",
        /// TPEG Pti26_255, undefined
        Undefined = "undefined",
    }
}

siri_enum! {
    /// Values for ScopeType - TPEG Pts36, AlertForType with additional values
    ScopeType as "ScopeTypeEnumeration" {
        /// TPEG Pts36_0, unknown
        Unknown = "unknown",
        /// TPEG Pts36_1, STOP PLACE
        StopPlace = "stopPlace",
        /// TPEG Pts36_2, line
        Line = "line",
        /// TPEG Pts36_3, route
        Route = "route",
        /// TPEG Pts36_4, individual PT service
        PublicTransportService = "publicTransportService",
        /// TPEG Pts36_5, operator
        Operator = "operator",
        /// TPEG Pts36_6, city
        City = "city",
        /// TPEG Pts36_7, area
        Area = "area",
        /// TPEG Pts36_8, stop point
        StopPoint = "stopPoint",
        /// STOP PLACE component
        StopPlaceComponent = "stopPlaceComponent",
        /// place
        Place = "place",
        /// network
        Network = "network",
        /// vehicle journey
        VehicleJourney = "vehicleJourney",
        /// dated vehicle journey
        DatedVehicleJourney = "datedVehicleJourney",
        /// connection link
        ConnectionLink = "connectionLink",
        /// interchange
        Interchange = "interchange",
        /// TPEG Pts36_0, unknown
        AllPT = "allPT",
        /// general
        General = "general",
        /// road
        Road = "road",
        /// TPEG Pts36_255, undefined
        Undefined = "undefined",
    }
}

siri_enum! {
    /// Values for Predictability Status.
    Predictability as "PredictabilityEnumeration" {
        /// planned
        Planned = "planned",
        /// unplanned
        Unplanned = "unplanned",
        /// all
        All = "all",
    }
}

siri_enum! {
    /// Values for TPEG Pti032 - VerificationStatus
    VerificationStatus as "VerificationStatusEnumeration" {
        /// TPEG Pti32_0, unknown
        Unknown = "unknown",
        /// TPEG Pti32_1, unverified
        Unverified = "unverified",
        /// TPEG Pti32_?, verified
        Verified = "verified",
        /// TPEG Pti32_?, verifiedAsDuplicate
        VerifiedAsDuplicate = "verifiedAsDuplicate",
        /// TPEG Pti32_255 ?
        Undefined = "undefined",
    }
}

siri_enum! {
    /// Values for Entry Status.
    WorkflowStatus as "WorkflowStatusEnumeration" {
        /// draft
        Draft = "draft",
        /// pendingApproval
        PendingApproval = "pendingApproval",
        /// approvedDraft
        ApprovedDraft = "approvedDraft",
        /// open
        Open = "open",
        /// published
        Published = "published",
        /// closing
        Closing = "closing",
        /// closed
        Closed = "closed",
    }
}

siri_enum! {
    /// Status of the related information (i.e. real, test or exercise).
    InformationStatus as "InformationStatusEnum" {
        /// The information is real. It is not a test or exercise.
        Real = "real",
        /// The information is part of an exercise which is for testing security.
        SecurityExercise = "securityExercise",
        /// The information is part of an exercise which includes tests of associated technical subsystems.
        TechnicalExercise = "technicalExercise",
        /// The information is part of a test for checking the exchange of this type of information.
        Test = "test",
    }
}

siri_enum! {
    /// Allowed values for EndTime Precision
    EndTimePrecision as "EndTimePrecisionEnumeration" {
        /// day
        Day = "day",
        /// hour
        Hour = "hour",
        /// second
        Second = "second",
        /// milliSecond
        MilliSecond = "milliSecond",
    }
}

siri_enum! {
    /// List of general directions of travel.
    Direction as "DirectionEnum" {
        /// Anticlockwise direction of travel on a ring road.
        Anticlockwise = "anticlockwise",
        /// Clockwise direction of travel on a ring road.
        Clockwise = "clockwise",
        /// North bound direction of travel.
        NorthBound = "northBound",
        /// North east bound direction of travel.
        NorthEastBound = "northEastBound",
        /// East bound direction of travel.
        EastBound = "eastBound",
        /// South east bound direction of travel.
        SouthEastBound = "southEastBound",
        /// South bound direction of travel.
        SouthBound = "southBound",
        /// South west bound direction of travel.
        SouthWestBound = "southWestBound",
        /// West bound direction of travel.
        WestBound = "westBound",
        /// North west bound direction of travel.
        NorthWestBound = "northWestBound",
        /// Heading towards town centre direction of travel.
        InboundTowardsTown = "inboundTowardsTown",
        /// Heading out of or away from the town centre direction of travel.
        OutboundFromTown = "outboundFromTown",
    }
}

siri_enum! {
    /// Identification of mobility USER NEEDs.
    Mobility as "MobilityEnumeration" {
        /// wheelchair
        Wheelchair = "wheelchair",
        /// assistedWheelchair
        AssistedWheelchair = "assistedWheelchair",
        /// motorizedWheelchair
        MotorizedWheelchair = "motorizedWheelchair",
        /// walkingFrame
        WalkingFrame = "walkingFrame",
        /// restrictedMobility
        RestrictedMobility = "restrictedMobility",
        /// otherMobilityNeed
        OtherMobilityNeed = "otherMobilityNeed",
    }
}

siri_enum! {
    /// Enumeration of specific psychosensory USER NEEDs.
    PyschosensoryNeed as "PyschosensoryNeedEnumeration" {
        /// visualImpairment
        VisualImpairment = "visualImpairment",
        /// auditoryImpairment
        AuditoryImpairment = "auditoryImpairment",
        /// cognitiveInputImpairment
        CognitiveInputImpairment = "cognitiveInputImpairment",
        /// averseToLifts
        AverseToLifts = "averseToLifts",
        /// averseToEscalators
        AverseToEscalators = "averseToEscalators",
        /// averseToConfinedSpaces
        AverseToConfinedSpaces = "averseToConfinedSpaces",
        /// averseToCrowds
        AverseToCrowds = "averseToCrowds",
        /// otherPsychosensoryNeed
        OtherPsychosensoryNeed = "otherPsychosensoryNeed",
    }
}

siri_enum! {
    /// Enumeration of specific encumbrances USER NEEDs.
    Encumbrance as "EncumbranceEnumeration" {
        /// luggageEncumbered
        LuggageEncumbered = "luggageEncumbered",
        /// pushchair
        Pushchair = "pushchair",
        /// baggageTrolley
        BaggageTrolley = "baggageTrolley",
        /// oversizeBaggage
        OversizeBaggage = "oversizeBaggage",
        /// guideDog
        GuideDog = "guideDog",
        /// otherAnimal
        OtherAnimal = "otherAnimal",
        /// otherEncumbrance
        OtherEncumbrance = "otherEncumbrance",
    }
}

siri_enum! {
    /// Enumeration of specific Medical USER NEEDs.
    MedicalNeed as "MedicalNeedEnumeration" {
        /// allergic
        Allergic = "allergic",
        /// heartCondition
        HeartCondition = "heartCondition",
        /// otherMedicalNeed
        OtherMedicalNeed = "otherMedicalNeed",
    }
}

siri_enum! {
    /// Values for TPEG Pts38 - AlertCause, plus additional values from TPEG Pti19/20/21/22
    AlertCause as "AlertCauseEnumeration" {
        /// TPEG Pts38_0, unknown
        Unknown = "unknown",
        /// TPEG Pts38_1, security alert
        SecurityAlert = "securityAlert",
        /// TPEG Pts38_2, emergency services call
        EmergencyServicesCall = "emergencyServicesCall",
        /// TPEG Pts38_3, police activity
        PoliceActivity = "policeActivity",
        /// TPEG Pts38_4, police order
        PoliceOrder = "policeOrder",
        /// TPEG Pts38_5, fire
        Fire = "fire",
        /// TPEG Pts38_6, cable fire
        CableFire = "cableFire",
        /// TPEG Pts38_7, smoke detected on vehicle
        SmokeDetectedOnVehicle = "smokeDetectedOnVehicle",
        /// TPEG Pts38_8, fire at the station
        FireAtStation = "fireAtStation",
        /// TPEG Pts38_9, fire run
        FireRun = "fireRun",
        /// TPEG Pts38_10, fire brigade order
        FireBrigadeOrder = "fireBrigadeOrder",
        /// TPEG Pts38_11, explosion
        Explosion = "explosion",
        /// TPEG Pts38_12, explosion hazard
        ExplosionHazard = "explosionHazard",
        /// TPEG Pts38_13, bomb disposal
        BombDisposal = "bombDisposal",
        /// TPEG Pts38_14, emergency medical services
        EmergencyMedicalServices = "emergencyMedicalServices",
        /// TPEG Pts38_15, emergency brake
        EmergencyBrake = "emergencyBrake",
        /// TPEG Pts38_16, vandalism
        Vandalism = "vandalism",
        /// TPEG Pts38_17, cable theft
        CableTheft = "cableTheft",
        /// TPEG Pts38_18, signal passed at danger
        SignalPassedAtDanger = "signalPassedAtDanger",
        /// TPEG Pts38_19, station overrun
        StationOverrun = "stationOverrun",
        /// TPEG Pts38_20, passengers blocking doors
        PassengersBlockingDoors = "passengersBlockingDoors",
        /// TPEG Pts38_21, defective security system
        DefectiveSecuritySystem = "defectiveSecuritySystem",
        /// TPEG Pts38_22, overcrowded
        Overcrowded = "overcrowded",
        /// TPEG Pts38_23, border control
        BorderControl = "borderControl",
        /// TPEG Pts38_24, unattended bag
        UnattendedBag = "unattendedBag",
        /// TPEG Pts38_25, telephoned threat
        TelephonedThreat = "telephonedThreat",
        /// TPEG Pts38_26, suspect vehicle
        SuspectVehicle = "suspectVehicle",
        /// TPEG Pts38_27, evacuation
        Evacuation = "evacuation",
        /// TPEG Pts38_28, terrorist incident
        TerroristIncident = "terroristIncident",
        /// TPEG Pts38_29, public disturbance
        PublicDisturbance = "publicDisturbance",
        /// TPEG Pts38_30, technical problem
        TechnicalProblem = "technicalProblem",
        /// TPEG Pts38_31, vehicle failure
        VehicleFailure = "vehicleFailure",
        /// TPEG Pts38_32, service disruption
        ServiceDisruption = "serviceDisruption",
        /// TPEG Pts38_33, door failure
        DoorFailure = "doorFailure",
        /// TPEG Pts38_34, lighting failure
        LightingFailure = "lightingFailure",
        /// TPEG Pts38_35, points problem
        PointsProblem = "pointsProblem",
        /// TPEG Pts38_36, points failure
        PointsFailure = "pointsFailure",
        /// TPEG Pts38_37, signal problem
        SignalProblem = "signalProblem",
        /// TPEG Pts38_38, signal failure
        SignalFailure = "signalFailure",
        /// TPEG Pts38_39, overhead wire failure
        OverheadWireFailure = "overheadWireFailure",
        /// TPEG Pts38_40, level crossing failure
        LevelCrossingFailure = "levelCrossingFailure",
        /// TPEG Pts38_41, traffic management system failure
        TrafficManagementSystemFailure = "trafficManagementSystemFailure",
        /// TPEG Pts38_42, engine failure
        EngineFailure = "engineFailure",
        /// TPEG Pts38_43, break down
        BreakDown = "breakDown",
        /// TPEG Pts38_44, repair work
        RepairWork = "repairWork",
        /// TPEG Pts38_45, construction work
        ConstructionWork = "constructionWork",
        /// TPEG Pts38_46, maintenance work
        MaintenanceWork = "maintenanceWork",
        /// TPEG Pts38_47, power problem
        PowerProblem = "powerProblem",
        /// TPEG Pts38_48, track circuit
        TrackCircuitProblem = "trackCircuitProblem",
        /// TPEG Pts38_49, swing bridge failure
        SwingBridgeFailure = "swingBridgeFailure",
        /// TPEG Pts38_50, escalator failure
        EscalatorFailure = "escalatorFailure",
        /// TPEG Pts38_51, lift failure
        LiftFailure = "liftFailure",
        /// TPEG Pts38_52, gangway problem
        GangwayProblem = "gangwayProblem",
        /// TPEG Pts38_53, defective vehicle
        DefectiveVehicle = "defectiveVehicle",
        /// TPEG Pts38_54, broken rail
        BrokenRail = "brokenRail",
        /// TPEG Pts38_55, poor rail conditions
        PoorRailConditions = "poorRailConditions",
        /// TPEG Pts38_56, de-icing work
        DeicingWork = "deicingWork",
        /// TPEG Pts38_57, wheel problem
        WheelProblem = "wheelProblem",
        /// TPEG Pts38_58, route blockage
        RouteBlockage = "routeBlockage",
        /// TPEG Pts38_59, congestion
        Congestion = "congestion",
        /// TPEG Pts38_60, heavy traffic
        HeavyTraffic = "heavyTraffic",
        /// TPEG Pts38_61, route diversion
        RouteDiversion = "routeDiversion",
        /// TPEG Pts38_62, roadworks
        Roadworks = "roadworks",
        /// TPEG Pts38_63, unscheduled construction work
        UnscheduledConstructionWork = "unscheduledConstructionWork",
        /// TPEG Pts38_64, level crossing incident
        LevelCrossingIncident = "levelCrossingIncident",
        /// TPEG Pts38_65, sewerageMaintenance
        SewerageMaintenance = "sewerageMaintenance",
        /// TPEG Pts38_66, road closed
        RoadClosed = "roadClosed",
        /// TPEG Pts38_67, roadway damage
        RoadwayDamage = "roadwayDamage",
        /// TPEG Pts38_68, bridge damage
        BridgeDamage = "bridgeDamage",
        /// TPEG Pts38_69, person on the line
        PersonOnTheLine = "personOnTheLine",
        /// TPEG Pts38_70, object on the line
        ObjectOnTheLine = "objectOnTheLine",
        /// TPEG Pts38_71, vehicle on the line
        VehicleOnTheLine = "vehicleOnTheLine",
        /// TPEG Pts38_72, animal on the line
        AnimalOnTheLine = "animalOnTheLine",
        /// TPEG Pts38_73, fallen tree on the line
        FallenTreeOnTheLine = "fallenTreeOnTheLine",
        /// TPEG Pts38_74, vegetation
        Vegetation = "vegetation",
        /// TPEG Pts38_75, speed restrictions
        SpeedRestrictions = "speedRestrictions",
        /// TPEG Pts38_76, preceding vehicle
        PrecedingVehicle = "precedingVehicle",
        /// TPEG Pts38_77, accident
        Accident = "accident",
        /// TPEG Pts38_78, near miss
        NearMiss = "nearMiss",
        /// TPEG Pts38_79, person hit by vehicle
        PersonHitByVehicle = "personHitByVehicle",
        /// TPEG Pts38_80, vehicle struck object
        VehicleStruckObject = "vehicleStruckObject",
        /// TPEG Pts38_81, vehicle struck animal
        VehicleStruckAnimal = "vehicleStruckAnimal",
        /// TPEG Pts38_82, derailment
        Derailment = "derailment",
        /// TPEG Pts38_83, collision
        Collision = "collision",
        /// TPEG Pts38_84, level crossing accident
        LevelCrossingAccident = "levelCrossingAccident",
        /// TPEG Pts38_85, poor weather
        PoorWeather = "poorWeather",
        /// TPEG Pts38_86, fog
        Fog = "fog",
        /// TPEG Pts38_87, heavy snowfall
        HeavySnowFall = "heavySnowFall",
        /// TPEG Pts38_88, heavy rain
        HeavyRain = "heavyRain",
        /// TPEG Pts38_89, strong winds
        StrongWinds = "strongWinds",
        /// TPEG Pts38_90, ice
        Ice = "ice",
        /// TPEG Pts38_91, hail
        Hail = "hail",
        /// TPEG Pts38_92, high temperatures
        HighTemperatures = "highTemperatures",
        /// TPEG Pts38_93, flooding
        Flooding = "flooding",
        /// TPEG Pts38_94, low water level
        LowWaterLevel = "lowWaterLevel",
        /// TPEG Pts38_95, risk of flooding
        RiskOfFlooding = "riskOfFlooding",
        /// TPEG Pts38_96, high water level
        HighWaterLevel = "highWaterLevel",
        /// TPEG Pts38_97, fallen leaves
        FallenLeaves = "fallenLeaves",
        /// TPEG Pts38_98, fallen tree
        FallenTree = "fallenTree",
        /// TPEG Pts38_99, landslide
        Landslide = "landslide",
        /// TPEG Pts38_100, risk of landslide
        RiskOfLandslide = "riskOfLandslide",
        /// TPEG Pts38_101, drifting snow
        DriftingSnow = "driftingSnow",
        /// TPEG Pts38_102, blizzard conditions
        BlizzardConditions = "blizzardConditions",
        /// TPEG Pts38_103, storm damage
        StormDamage = "stormDamage",
        /// TPEG Pts38_104, lightning strike
        LightningStrike = "lightningStrike",
        /// TPEG Pts38_105, rough sea
        RoughSea = "roughSea",
        /// TPEG Pts38_106, high tide
        HighTide = "highTide",
        /// TPEG Pts38_107, low tide
        LowTide = "lowTide",
        /// TPEG Pts38_108, ice drift
        IceDrift = "iceDrift",
        /// TPEG Pts38_109, avalanches
        Avalanches = "avalanches",
        /// TPEG Pts38_110, risk of avalanches
        RiskOfAvalanches = "riskOfAvalanches",
        /// TPEG Pts38_111, flash floods
        FlashFloods = "flashFloods",
        /// TPEG Pts38_112, mudslide
        Mudslide = "mudslide",
        /// TPEG Pts38_113, rockfalls
        Rockfalls = "rockfalls",
        /// TPEG Pts38_114, subsidence
        Subsidence = "subsidence",
        /// TPEG Pts38_115, earthquake damage
        EarthquakeDamage = "earthquakeDamage",
        /// TPEG Pts38_116, grass fire
        GrassFire = "grassFire",
        /// TPEG Pts38_117, wildland fire
        WildlandFire = "wildlandFire",
        /// TPEG Pts38_118, ice on railway
        IceOnRailway = "iceOnRailway",
        /// TPEG Pts38_119, ice on carriages
        IceOnCarriages = "iceOnCarriages",
        /// TPEG Pts38_120, special event
        SpecialEvent = "specialEvent",
        /// TPEG Pts38_121, procession
        Procession = "procession",
        /// TPEG Pts38_122, demonstration
        Demonstration = "demonstration",
        /// TPEG Pts38_123, industrial action
        IndustrialAction = "industrialAction",
        /// TPEG Pts38_124, staff sickness
        StaffSickness = "staffSickness",
        /// TPEG Pts38_125, staff absence
        StaffAbsence = "staffAbsence",
        /// TPEG Pts38_126, operator ceased trading
        OperatorCeasedTrading = "operatorCeasedTrading",
        /// TPEG Pts38_127, previous disturbances
        PreviousDisturbances = "previousDisturbances",
        /// TPEG Pts38_128, vehicle blocking track
        VehicleBlockingTrack = "vehicleBlockingTrack",
        /// TPEG Pts38_129, foreign disturbances
        ForeignDisturbances = "foreignDisturbances",
        /// TPEG Pts38_130, awaiting shuttle
        AwaitingShuttle = "awaitingShuttle",
        /// TPEG Pts38_131, change in carriages
        ChangeInCarriages = "changeInCarriages",
        /// TPEG Pts38_132, train coupling
        TrainCoupling = "trainCoupling",
        /// TPEG Pts38_133, boarding delay
        BoardingDelay = "boardingDelay",
        /// TPEG Pts38_134, awaiting approach
        AwaitingApproach = "awaitingApproach",
        /// TPEG Pts38_135, overtaking
        Overtaking = "overtaking",
        /// TPEG Pts38_136, provision delay
        ProvisionDelay = "provisionDelay",
        /// TPEG Pts38_137, miscellaneous
        Miscellaneous = "miscellaneous",
        /// TPEG Pts38_255, undefined alert cause
        UndefinedAlertCause = "undefinedAlertCause",
        /// TPEG Pti19_1, DEPRECATED since SIRI 2.1
        Incident = "incident",
        /// TPEG Pti19_1_2, DEPRECATED since SIRI 2.1
        SafetyViolation = "safetyViolation",
        /// TPEG Pti19_1_5, DEPRECATED since SIRI 2.1 - replaced by Pts38_33, door failure
        TrainDoor = "trainDoor",
        /// TPEG Pti19_1_7, DEPRECATED since SIRI 2.1
        Altercation = "altercation",
        /// TPEG Pti19_1_8, DEPRECATED since SIRI 2.1
        IllVehicleOccupants = "illVehicleOccupants",
        /// TPEG Pti19_1_12, DEPRECATED since SIRI 2.1 - replaced by Pts38_32, service disruption
        ServiceFailure = "serviceFailure",
        /// TPEG Pti19_2, DEPRECATED since SIRI 2.1
        BombExplosion = "bombExplosion",
        /// TPEG Pti19_3_2, DEPRECATED since SIRI 2.1
        FireBrigadeSafetyChecks = "fireBrigadeSafetyChecks",
        /// TPEG Pti19_3_6, DEPRECATED since SIRI 2.1
        CivilEmergency = "civilEmergency",
        /// TPEG Pti19_3_7, DEPRECATED since SIRI 2.1
        AirRaid = "airRaid",
        /// TPEG Pti19_3_8, DEPRECATED since SIRI 2.1
        Sabotage = "sabotage",
        /// TPEG Pti19_3_9, DEPRECATED since SIRI 2.1
        BombAlert = "bombAlert",
        /// TPEG Pti19_3_10, DEPRECATED since SIRI 2.1
        Attack = "attack",
        /// TPEG Pti19_3_13, DEPRECATED since SIRI 2.1
        GunfireOnRoadway = "gunfireOnRoadway",
        /// TPEG Pti19_3_16, DEPRECATED since SIRI 2.1
        SecurityIncident = "securityIncident",
        /// TPEG Pti19_4_1, DEPRECATED since SIRI 2.1
        LinesideFire = "linesideFire",
        /// TPEG Pti19_5_1, DEPRECATED since SIRI 2.1
        PassengerAction = "passengerAction",
        /// TPEG Pti19_5_2, DEPRECATED since SIRI 2.1
        StaffAssault = "staffAssault",
        /// TPEG Pti19_5_3, DEPRECATED since SIRI 2.1
        RailwayCrime = "railwayCrime",
        /// TPEG Pti19_5_4, DEPRECATED since SIRI 2.1
        Assault = "assault",
        /// TPEG Pti19_5_5, DEPRECATED since SIRI 2.1
        Theft = "theft",
        /// TPEG Pti19_6_1, DEPRECATED since SIRI 2.1
        Fatality = "fatality",
        /// TPEG Pti19_6_2, DEPRECATED since SIRI 2.1
        PersonUnderTrain = "personUnderTrain",
        /// TPEG Pti19_6_3, DEPRECATED since SIRI 2.1 - replaced by Pts38_79, person hit by vehicle
        PersonHitByTrain = "personHitByTrain",
        /// TPEG Pti19_6_4, DEPRECATED since SIRI 2.1
        PersonIllOnVehicle = "personIllOnVehicle",
        /// TPEG Pti19_6_5, DEPRECATED since SIRI 2.1 - replaced by Pts38_14, emergency medical services
        EmergencyServices = "emergencyServices",
        /// TPEG Pti19_8, DEPRECATED since SIRI 2.1
        InsufficientDemand = "insufficientDemand",
        /// TPEG Pti19_10, DEPRECATED since SIRI 2.1
        LeaderBoardFailure = "leaderBoardFailure",
        /// TPEG Pti19_11, DEPRECATED since SIRI 2.1
        ServiceIndicatorFailure = "serviceIndicatorFailure",
        /// TPEG Pti19_14, DEPRECATED since SIRI 2.1
        OperatorSuspended = "operatorSuspended",
        /// TPEG Pti19_15_1, DEPRECATED since SIRI 2.1 - replaced by Pts38_23, border control
        ProblemsAtBorderPost = "problemsAtBorderPost",
        /// TPEG Pti19_15_2, DEPRECATED since SIRI 2.1
        ProblemsAtCustomsPost = "problemsAtCustomsPost",
        /// TPEG Pti19_19_3, DEPRECATED since SIRI 2.1 - replaced by Pts38_81, vehicle struck animal
        TrainStruckAnimal = "trainStruckAnimal",
        /// TPEG Pti19_19_4, DEPRECATED since SIRI 2.1 - replaced by Pts38_80, vehicle struck object
        TrainStruckObject = "trainStruckObject",
        /// TPEG Pti19_23_2, DEPRECATED since SIRI 2.1
        RoadMaintenance = "roadMaintenance",
        /// TPEG Pti19_23_3, DEPRECATED since SIRI 2.1
        Asphalting = "asphalting",
        /// TPEG Pti19_23_4, DEPRECATED since SIRI 2.1
        Paving = "paving",
        /// TPEG Pti19_24_1, DEPRECATED since SIRI 2.1
        March = "march",
        /// TPEG Pti19_24_5, DEPRECATED since SIRI 2.1
        FilterBlockade = "filterBlockade",
        /// TPEG Pti19_24_6, DEPRECATED since SIRI 2.1
        SightseersObstructingAccess = "sightseersObstructingAccess",
        /// TPEG Pti19_24_7, DEPRECATED since SIRI 2.1
        Holiday = "holiday",
        /// TPEG Pti19_25, DEPRECATED since SIRI 2.1 - replaced by Pts38_68, bridge damage
        BridgeStrike = "bridgeStrike",
        /// TPEG Pti19_25_1, DEPRECATED since SIRI 2.1
        ViaductFailure = "viaductFailure",
        /// TPEG Pti19_26, DEPRECATED since SIRI 2.1
        OverheadObstruction = "overheadObstruction",
        /// TPEG Pti19_255, DEPRECATED since SIRI 2.1
        UndefinedProblem = "undefinedProblem",
        /// TPEG Pti19_255_1, DEPRECATED since SIRI 2.1
        LogisticProblems = "logisticProblems",
        /// TPEG Pti19_255_2, DEPRECATED since SIRI 2.1
        ProblemsOnLocalRoad = "problemsOnLocalRoad",
        /// TPEG Pti20_1_1, DEPRECATED since SIRI 2.1
        StaffInjury = "staffInjury",
        /// TPEG Pti20_1_2, DEPRECATED since SIRI 2.1
        ContractorStaffInjury = "contractorStaffInjury",
        /// TPEG Pti20_3, DEPRECATED since SIRI 2.1
        StaffInWrongPlace = "staffInWrongPlace",
        /// TPEG Pti20_4, DEPRECATED since SIRI 2.1
        StaffShortage = "staffShortage",
        /// TPEG Pti20_5_1, DEPRECATED since SIRI 2.1
        UnofficialIndustrialAction = "unofficialIndustrialAction",
        /// TPEG Pti20_6, DEPRECATED since SIRI 2.1
        WorkToRule = "workToRule",
        /// TPEG Pti20_255, DEPRECATED since SIRI 2.1 - replaced by Pts38_255, undefined alert cause
        UndefinedPersonnelProblem = "undefinedPersonnelProblem",
        /// TPEG Pti21_3_1, DEPRECATED since SIRI 2.1
        TrainWarningSystemProblem = "trainWarningSystemProblem",
        /// TPEG Pti21_4_1, DEPRECATED since SIRI 2.1
        SignalAndSwitchFailure = "signalAndSwitchFailure",
        /// TPEG Pti21_6_1, DEPRECATED since SIRI 2.1
        TractionFailure = "tractionFailure",
        /// TPEG Pti21_6_2, DEPRECATED since SIRI 2.1 - replaced by Pts38_53, defective vehicle
        DefectiveTrain = "defectiveTrain",
        /// TPEG Pti21_8_3, DEPRECATED since SIRI 2.1
        WheelImpactLoad = "wheelImpactLoad",
        /// TPEG Pti21_8_4, DEPRECATED since SIRI 2.1
        LackOfOperationalStock = "lackOfOperationalStock",
        /// TPEG Pti21_8_5, DEPRECATED since SIRI 2.1
        DefectiveFireAlarmEquipment = "defectiveFireAlarmEquipment",
        /// TPEG Pti21_8_6, DEPRECATED since SIRI 2.1
        DefectivePlatformEdgeDoors = "defectivePlatformEdgeDoors",
        /// TPEG Pti21_8_7, DEPRECATED since SIRI 2.1
        DefectiveCctv = "defectiveCctv",
        /// TPEG Pti21_8_8, DEPRECATED since SIRI 2.1
        DefectivePublicAnnouncementSystem = "defectivePublicAnnouncementSystem",
        /// TPEG Pti21_8_9, DEPRECATED since SIRI 2.1
        TicketingSystemNotAvailable = "ticketingSystemNotAvailable",
        /// TPEG Pti21_11_1, DEPRECATED since SIRI 2.1
        EmergencyEngineeringWork = "emergencyEngineeringWork",
        /// TPEG Pti21_11_2, DEPRECATED since SIRI 2.1
        LateFinishToEngineeringWork = "lateFinishToEngineeringWork",
        /// TPEG Pti21_13, DEPRECATED since SIRI 2.1
        FuelProblem = "fuelProblem",
        /// TPEG Pti21_18, DEPRECATED since SIRI 2.1
        ClosedForMaintenance = "closedForMaintenance",
        /// TPEG Pti21_19, DEPRECATED since SIRI 2.1
        FuelShortage = "fuelShortage",
        /// TPEG Pti21_21_1, DEPRECATED since SIRI 2.1
        SlipperyTrack = "slipperyTrack",
        /// TPEG Pti21_22, DEPRECATED since SIRI 2.1
        LuggageCarouselProblem = "luggageCarouselProblem",
        /// TPEG Pti21_255, DEPRECATED since SIRI 2.1 - replaced by Pts38_255, undefined alert cause
        UndefinedEquipmentProblem = "undefinedEquipmentProblem",
        /// TPEG Pti22_5_1, DEPRECATED since SIRI 2.1
        StormConditions = "stormConditions",
        /// TPEG Pti22_6, DEPRECATED since SIRI 2.1
        TidalRestrictions = "tidalRestrictions",
        /// TPEG Pti22_9_1, DEPRECATED since SIRI 2.1
        Slipperiness = "slipperiness",
        /// TPEG Pti22_9_3, DEPRECATED since SIRI 2.1
        GlazedFrost = "glazedFrost",
        /// TPEG Pti22_10, DEPRECATED since SIRI 2.1
        Frozen = "frozen",
        /// TPEG Pti22_11_1, DEPRECATED since SIRI 2.1
        Sleet = "sleet",
        /// TPEG Pti22_14, DEPRECATED since SIRI 2.1
        Waterlogged = "waterlogged",
        /// TPEG Pti22_255_2, DEPRECATED since SIRI 2.1
        SewerOverflow = "sewerOverflow",
        /// TPEG Pti22_255, DEPRECATED since SIRI 2.1 - replaced by Pts38_255, undefined alert cause
        UndefinedEnvironmentalProblem = "undefinedEnvironmentalProblem",
        /// See also TPEG Pts38_8 value 'fireAtStation'.
        FireAtTheStation = "fireAtTheStation",
        /// See also TPEG Pts38_43 value 'breakDown'.
        Breakdown = "breakdown",
        /// See also TPEG Pts38_64 value 'levelCrossingIncident'.
        LevelCrossingBlocked = "levelCrossingBlocked",
        /// See also TPEG Pts38_87 value 'heavySnowFall'.
        HeavySnowfall = "heavySnowfall",
        /// See also TPEG Pts38_130 value 'awaitingShuttle'.
        WaitingForTransferPassengers = "waitingForTransferPassengers",
        /// See also TPEG Pts38_134 value 'awaitingApproach'.
        AwaitingOncomingVehicle = "awaitingOncomingVehicle",
    }
}

siri_enum! {
    /// Values for Audience.
    Audience as "AudienceEnumeration" {
        /// public
        Public = "public",
        /// emergencyServices
        EmergencyServices = "emergencyServices",
        /// staff
        Staff = "staff",
        /// stationStaff
        StationStaff = "stationStaff",
        /// management
        Management = "management",
        /// authorities
        Authorities = "authorities",
        /// infoServices
        InfoServices = "infoServices",
        /// transportOperators
        TransportOperators = "transportOperators",
    }
}

siri_enum! {
    /// Values for TPEG Pti34 - DayType
    DayType as "DayTypeEnumeration" {
        /// TPEG Pti34_0, unknown
        Unknown = "unknown",
        /// TPEG Pti34_1, Monday
        Monday = "monday",
        /// TPEG Pti34_2, Tuesday
        Tuesday = "tuesday",
        /// TPEG Pti34_3, Wednesday
        Wednesday = "wednesday",
        /// TPEG Pti34_4, Thursday
        Thursday = "thursday",
        /// TPEG Pti34_5, Friday
        Friday = "friday",
        /// TPEG Pti34_6, Saturday
        Saturday = "saturday",
        /// TPEG Pti34_7, Sunday
        Sunday = "sunday",
        /// TPEG Pti34_8, weekdays
        Weekdays = "weekdays",
        /// TPEG Pti34_9, weekends
        Weekends = "weekends",
        /// TPEG Pti34_10, holiday
        Holiday = "holiday",
        /// TPEG Pti34_11, public holiday
        PublicHoliday = "publicHoliday",
        /// TPEG Pti34_12, religious holiday
        ReligiousHoliday = "religiousHoliday",
        /// TPEG Pti34_13, federal holiday
        FederalHoliday = "federalHoliday",
        /// TPEG Pti34_14, regional holiday
        RegionalHoliday = "regionalHoliday",
        /// TPEG Pti34_15, national holiday
        NationalHoliday = "nationalHoliday",
        /// TPEG Pti34_16, Monday to Friday
        MondayToFriday = "mondayToFriday",
        /// TPEG Pti34_17, Monday to Saturday
        MondayToSaturday = "mondayToSaturday",
        /// TPEG Pti34_18, Sundays and public holidays
        SundaysAndPublicHolidays = "sundaysAndPublicHolidays",
        /// TPEG Pti34_19, school days
        SchoolDays = "schoolDays",
        /// TPEG Pti34_20, every day
        EveryDay = "everyDay",
        /// TPEG Pti34_255, undefined day type
        UndefinedDayType = "undefinedDayType",
    }
}

siri_enum! {
    /// Allowed values for EndTime Status.
    EndTimeStatus as "EndTimeStatusEnumeration" {
        /// undefined
        Undefined = "undefined",
        /// longTerm
        LongTerm = "longTerm",
        /// shortTerm
        ShortTerm = "shortTerm",
    }
}

siri_enum! {
    /// Values for image content.
    ImageContent as "ImageContentEnumeration" {
        /// map
        Map = "map",
        /// graphic
        Graphic = "graphic",
        /// logo
        Logo = "logo",
    }
}

siri_enum! {
    /// Values for image content.
    LinkContent as "LinkContentEnumeration" {
        /// timetable
        Timetable = "timetable",
        /// relatedSite
        RelatedSite = "relatedSite",
        /// details
        Details = "details",
        /// advice
        Advice = "advice",
        /// other
        Other = "other",
    }
}

siri_enum! {
    /// Levels of confidence that the sender has in the information, ordered {certain, probable, risk of}.
    ProbabilityOfOccurrence as "ProbabilityOfOccurrenceEnum" {
        /// The source is completely certain of the occurrence of the situation record version content.
        Certain = "certain",
        /// The source has a reasonably high level of confidence of the occurrence of the situation record version content.
        Probable = "probable",
        /// The source has a moderate level of confidence of the occurrence of the situation record version content.
        RiskOf = "riskOf",
    }
}

siri_enum! {
    /// Type of public event (Datex2 PublicEventTypeEnum and PublicEventType2Enum combined)
    PublicEventType as "PublicEventTypeEnum" {
        /// Unknown
        Unknown = "unknown",
        /// Agricultural show or event which could disrupt traffic.
        AgriculturalShow = "agriculturalShow",
        /// Air show or other aeronautical event which could disrupt traffic.
        AirShow = "airShow",
        /// Art event that could disrupt traffic.
        ArtEvent = "artEvent",
        /// Athletics event that could disrupt traffic.
        AthleticsMeeting = "athleticsMeeting",
        /// Beer festival that could disrupt traffic.
        BeerFestival = "beerFestival",
        /// Ball game event that could disrupt traffic.
        BallGame = "ballGame",
        /// Baseball game event that could disrupt traffic.
        BaseballGame = "baseballGame",
        /// Basketball game event that could disrupt traffic.
        BasketballGame = "basketballGame",
        /// Bicycle race that could disrupt traffic.
        BicycleRace = "bicycleRace",
        /// Regatta (boat race event of sailing, powerboat or rowing) that could disrupt traffic.
        BoatRace = "boatRace",
        /// Boat show which could disrupt traffic.
        BoatShow = "boatShow",
        /// Boxing event that could disrupt traffic.
        BoxingTournament = "boxingTournament",
        /// Bull fighting event that could disrupt traffic.
        BullFight = "bullFight",
        /// Formal or religious act, rite or ceremony that could disrupt traffic.
        CeremonialEvent = "ceremonialEvent",
        /// Commercial event which could disrupt traffic.
        CommercialEvent = "commercialEvent",
        /// Concert event that could disrupt traffic.
        Concert = "concert",
        /// Cricket match that could disrupt traffic.
        CricketMatch = "cricketMatch",
        /// Cultural event which could disrupt traffic.
        CulturalEvent = "culturalEvent",
        /// Major display or trade show which could disrupt traffic.
        Exhibition = "exhibition",
        /// Periodic (e.g. annual), often traditional, gathering for entertainment or trade promotion, which could disrupt traffic.
        Fair = "fair",
        /// Celebratory event or series of events which could disrupt traffic.
        Festival = "festival",
        /// Film festival that could disrupt traffic.
        FilmFestival = "filmFestival",
        /// Film or TV making event which could disrupt traffic.
        FilmTVMaking = "filmTVMaking",
        /// Fireworks display that could disrupt traffic.
        FireworkDisplay = "fireworkDisplay",
        /// Flower event that could disrupt traffic.
        FlowerEvent = "flowerEvent",
        /// Food festival that could disrupt traffic.
        FoodFestival = "foodFestival",
        /// Football match that could disrupt traffic.
        FootballMatch = "footballMatch",
        /// Periodic (e.g. annual), often traditional, gathering for entertainment, which could disrupt traffic.
        Funfair = "funfair",
        /// Gardening and/or flower show or event which could disrupt traffic.
        GardeningOrFlowerShow = "gardeningOrFlowerShow",
        /// Golf tournament event that could disrupt traffic.
        GolfTournament = "golfTournament",
        /// Hockey game event that could disrupt traffic.
        HockeyGame = "hockeyGame",
        /// Horse race meeting that could disrupt traffic.
        HorseRaceMeeting = "horseRaceMeeting",
        /// Large sporting event of an international nature that could disrupt traffic.
        InternationalSportsMeeting = "internationalSportsMeeting",
        /// Significant organised event either on or near the roadway which could disrupt traffic.
        MajorEvent = "majorEvent",
        /// Marathon, cross-country or road running event that could disrupt traffic.
        Marathon = "marathon",
        /// Periodic (e.g. weekly) gathering for buying and selling, which could disrupt traffic.
        Market = "market",
        /// Sports match of unspecified type that could disrupt traffic.
        Match = "match",
        /// Motor show which could disrupt traffic.
        MotorShow = "motorShow",
        /// Motor sport race meeting that could disrupt traffic.
        MotorSportRaceMeeting = "motorSportRaceMeeting",
        /// Open air concert that could disrupt traffic.
        OpenAirConcert = "openAirConcert",
        /// Formal display or organised procession which could disrupt traffic.
        Parade = "parade",
        /// An organised procession which could disrupt traffic.
        Procession = "procession",
        /// Race meeting (other than horse or motor sport) that could disrupt traffic.
        RaceMeeting = "raceMeeting",
        /// Rugby match that could disrupt traffic.
        RugbyMatch = "rugbyMatch",
        /// A series of significant organised events either on or near the roadway which could disrupt traffic.
        SeveralMajorEvents = "severalMajorEvents",
        /// Entertainment event that could disrupt traffic.
        Show = "show",
        /// Horse showing jumping and tournament event that could disrupt traffic.
        ShowJumping = "showJumping",
        /// Sound and light show that could disrupt traffic.
        SoundAndLightShow = "soundAndLightShow",
        /// Sports event of unspecified type that could disrupt traffic.
        SportsMeeting = "sportsMeeting",
        /// Public ceremony or visit of national or international significance which could disrupt traffic.
        StateOccasion = "stateOccasion",
        /// Street festival that could disrupt traffic.
        StreetFestival = "streetFestival",
        /// Tennis tournament that could disrupt traffic.
        TennisTournament = "tennisTournament",
        /// Theatrical event that could disrupt traffic.
        TheatricalEvent = "theatricalEvent",
        /// Sporting event or series of events of unspecified type lasting more than one day which could disrupt traffic.
        Tournament = "tournament",
        /// A periodic (e.g. annual), often traditional, gathering for trade promotion, which could disrupt traffic.
        TradeFair = "tradeFair",
        /// Water sports meeting that could disrupt traffic.
        WaterSportsMeeting = "waterSportsMeeting",
        /// Wine festival that could disrupt traffic.
        WineFestival = "wineFestival",
        /// Winter sports meeting or event (e.g. skiing, ski jumping, skating) that could disrupt traffic.
        WinterSportsMeeting = "winterSportsMeeting",
        /// Other than as defined in this enumeration.
        Other = "other",
    }
}

siri_enum! {
    /// Classification of the quality of the prediction of the CALL, according to a fixed list of values. This may reflect a presentation policy, for example CALLs less than one minute behind target time are stiull classified as on-time. Applications may use this to guide their own presentation of times.
    QualityIndex as "QualityIndexEnumeration" {
        /// Data is certain (1/5).
        Certain = "certain",
        /// Data has confidence level of very reliable (2/5).
        VeryReliable = "veryReliable",
        /// Data has confidence level of reliable (3/5).
        Reliable = "reliable",
        /// Data is thought to be reliable (4/5)
        ProbablyReliable = "probablyReliable",
        /// Data is unconfirmed (5/5).
        Unconfirmed = "unconfirmed",
    }
}

siri_enum! {
    /// Values for Type of Source.
    RelatedTo as "RelatedToEnumeration" {
        /// cause
        Cause = "cause",
        /// effect
        Effect = "effect",
        /// correctionTo
        CorrectionTo = "correctionTo",
        /// update
        Update = "update",
        /// supercedes
        Supercedes = "supercedes",
        /// supercededBy
        SupercededBy = "supercededBy",
        /// associated
        Associated = "associated",
    }
}

siri_enum! {
    /// Values for TPEG Pti27 - ReportType
    ReportType as "ReportTypeEnumeration" {
        /// TPEG Pti27_0, unknown
        Unknown = "unknown",
        /// TPEG Pti27_1, incident
        Incident = "incident",
        /// TPEG Pti27_1_1, general
        General = "general",
        /// TPEG Pti27_1_2, operator
        Operator = "operator",
        /// TPEG Pti27_1_3, network
        Network = "network",
        /// TPEG Pti27_2, station terminal
        StationTerminal = "stationTerminal",
        /// TPEG Pti27_2_1, stoppoint
        StopPoint = "stopPoint",
        /// TPEG Pti27_2_2, connection link
        ConnectionLink = "connectionLink",
        /// TPEG Pti27_2_3, point
        Point = "point",
        /// TPEG Pti27_3, route
        Route = "route",
        /// TPEG Pti27_4, individual service
        IndividualService = "individualService",
        /// TPEG Pti27_255, undefined type
        Undefined = "undefined",
    }
}

siri_enum! {
    /// Values for Sensitivity.
    Sensitivity as "SensitivityEnumeration" {
        /// veryHigh
        VeryHigh = "veryHigh",
        /// high
        High = "high",
        /// medium
        Medium = "medium",
        /// low
        Low = "low",
        /// veryLow
        VeryLow = "veryLow",
    }
}

siri_enum! {
    /// Values for Type of Source.
    SituationSourceType as "SituationSourceTypeEnumeration" {
        /// directReport
        DirectReport = "directReport",
        /// email
        Email = "email",
        /// phone
        Phone = "phone",
        /// fax
        Fax = "fax",
        /// post
        Post = "post",
        /// feed
        Feed = "feed",
        /// radio
        Radio = "radio",
        /// tv
        Tv = "tv",
        /// web
        Web = "web",
        /// pager
        Pager = "pager",
        /// text
        Text = "text",
        /// other
        Other = "other",
    }
}

siri_enum! {
    /// Type of sources from which situation information may be derived.
    SourceType as "SourceTypeEnum" {
        /// A patrol of an automobile club.
        AutomobileClubPatrol = "automobileClubPatrol",
        /// A camera observation (either still or video camera).
        CameraObservation = "cameraObservation",
        /// An operator of freight vehicles.
        FreightVehicleOperator = "freightVehicleOperator",
        /// A station dedicated to the monitoring of the road network by processing inductive loop information.
        InductionLoopMonitoringStation = "inductionLoopMonitoringStation",
        /// A station dedicated to the monitoring of the road network by processing infrared image information.
        InfraredMonitoringStation = "infraredMonitoringStation",
        /// A station dedicated to the monitoring of the road network by processing microwave information.
        MicrowaveMonitoringStation = "microwaveMonitoringStation",
        /// See also 'microwaveMonitoringStation'
        MicrowavedMonitoringStation = "microwavedMonitoringStation",
        /// A caller using a mobile telephone (who may or may not be on the road network).
        MobileTelephoneCaller = "mobileTelephoneCaller",
        /// Emergency service patrols other than police.
        NonPoliceEmergencyServicePatrol = "nonPoliceEmergencyServicePatrol",
        /// See also 'nonPoliceEmergencyServicePatrol'
        NonPoliceEmergencyServicesPatrol = "nonPoliceEmergencyServicesPatrol",
        /// Other sources of information.
        OtherInformation = "otherInformation",
        /// Personnel from a vehicle belonging to the road operator or authority or any emergency service, including authorised breakdown service organisations.
        OtherOfficialVehicle = "otherOfficialVehicle",
        /// A police patrol.
        PolicePatrol = "policePatrol",
        /// A private breakdown service.
        PrivateBreakdownService = "privateBreakdownService",
        /// A utility organisation, either public or private.
        PublicAndPrivateUtilities = "publicAndPrivateUtilities",
        /// A motorist who is an officially registered observer.
        RegisteredMotoristObserver = "registeredMotoristObserver",
        /// See also 'registeredMotoristObserver'
        RegisteredMobileObserver = "registeredMobileObserver",
        /// A road authority.
        RoadAuthorities = "roadAuthorities",
        /// A patrol of the road operator or authority.
        RoadOperatorPatrol = "roadOperatorPatrol",
        /// A caller who is using an emergency roadside telephone.
        RoadsideTelephoneCaller = "roadsideTelephoneCaller",
        /// A spotter aircraft of an organisation specifically assigned to the monitoring of the traffic network.
        SpotterAircraft = "spotterAircraft",
        /// A station, usually automatic, dedicated to the monitoring of the road network.
        TrafficMonitoringStation = "trafficMonitoringStation",
        /// An operator of a transit service, e.g. bus link operator.
        TransitOperator = "transitOperator",
        /// A specially equipped vehicle used to provide measurements.
        VehicleProbeMeasurement = "vehicleProbeMeasurement",
        /// A station dedicated to the monitoring of the road network by processing video image information.
        VideoProcessingMonitoringStation = "videoProcessingMonitoringStation",
    }
}

siri_enum! {
    /// Identification of specific SUITABILITY.
    Suitability as "SuitabilityEnumeration" {
        /// suitable
        Suitable = "suitable",
        /// notSuitable
        NotSuitable = "notSuitable",
    }
}

siri_enum! {
    /// Values for Progress Status.
    ActionStatus as "ActionStatusEnumeration" {
        /// open
        Open = "open",
        /// published
        Published = "published",
        /// closed
        Closed = "closed",
    }
}

siri_enum! {
    /// Values for perspectives.
    Perspective as "PerspectiveEnumeration" {
        /// general
        General = "general",
        /// stopPoint
        StopPoint = "stopPoint",
        /// vehicleJourney
        VehicleJourney = "vehicleJourney",
    }
}

siri_enum! {
    /// Values for TPEG Pts039 - AdviceType, with some additional values
    AdviceType as "AdviceTypeEnumeration" {
        /// TPEG Pts39_0, unknown
        Unknown = "unknown",
        /// TPEG Pts39_1, use replacement bus
        UseReplacementBus = "useReplacementBus",
        /// TPEG Pts39_2, use replacement train
        UseReplacementTrain = "useReplacementTrain",
        /// TPEG Pts39_3, use the alternative route
        UseAlternativeRoute = "useAlternativeRoute",
        /// TPEG Pts39_4, go on foot
        GoOnFoot = "goOnFoot",
        /// TPEG Pts39_5, please leave the station! Danger!
        DangerLeaveStation = "dangerLeaveStation",
        /// TPEG Pts39_6, no means of travel
        NoMeansOfTravel = "noMeansOfTravel",
        /// TPEG Pts39_7, use different stops
        UseDifferentStops = "useDifferentStops",
        /// TPEG Pts39_8, use alternative stop
        UseAlternativeStop = "useAlternativeStop",
        /// TPEG Pts39_9, do not leave vehicle! Danger!
        DangerDoNotLeaveVehicle = "dangerDoNotLeaveVehicle",
        /// TPEG Pts39_10, take advice from announcements
        TakeAdviceAnnouncements = "takeAdviceAnnouncements",
        /// TPEG Pts39_11, take advice from personnel
        TakeAdvicePersonnel = "takeAdvicePersonnel",
        /// TPEG Pts39_12, obey advice from police
        ObeyAdvicePolice = "obeyAdvicePolice",
        /// use other PT services
        UseOtherPT = "useOtherPT",
        /// use interchange
        UseInterchange = "useInterchange",
        /// no advice
        NoAdvice = "noAdvice",
        /// TPEG Pts39_255, undefined advice
        UndefinedAdvice = "undefinedAdvice",
        /// take detour
        TakeDetour = "takeDetour",
        /// change accessibility
        UseAlternativeAccess = "useAlternativeAccess",
    }
}

siri_enum! {
    /// Allowed types activity for Alighting.
    ArrivalBoardingActivity as "ArrivalBoardingActivityEnumeration" {
        /// alighting
        Alighting = "alighting",
        /// noAlighting
        NoAlighting = "noAlighting",
        /// passThru
        PassThru = "passThru",
    }
}

siri_enum! {
    /// Allowed types activity for Boarding.
    DepartureBoardingActivity as "DepartureBoardingActivityEnumeration" {
        /// boarding
        Boarding = "boarding",
        /// noBoarding
        NoBoarding = "noBoarding",
        /// passThru
        PassThru = "passThru",
    }
}

siri_enum! {
    /// Type for allowed values of DelayBand. Based on Datex2, with some additional values.
    DelayBand as "DelayBandEnumeration" {
        /// delayTwoMinutes
        DelayTwoMinutes = "delayTwoMinutes",
        /// upToThreeMinutes
        UpToThreeMinutes = "upToThreeMinutes",
        /// upToFourMinutes
        UpToFourMinutes = "upToFourMinutes",
        /// upToFiveMinutes
        UpToFiveMinutes = "upToFiveMinutes",
        /// upToEightMinutes
        UpToEightMinutes = "upToEightMinutes",
        /// negligible
        Negligible = "negligible",
        /// upToTenMinutes
        UpToTenMinutes = "upToTenMinutes",
        /// betweenTenMinutesAndThirtyMinutes
        BetweenTenMinutesAndThirtyMinutes = "betweenTenMinutesAndThirtyMinutes",
        /// betweenThirtyMinutesAndOneHour
        BetweenThirtyMinutesAndOneHour = "betweenThirtyMinutesAndOneHour",
        /// betweenOneHourAndThreeHours
        BetweenOneHourAndThreeHours = "betweenOneHourAndThreeHours",
        /// betweenThreeHoursAndSixHours
        BetweenThreeHoursAndSixHours = "betweenThreeHoursAndSixHours",
        /// longerThanSixHours
        LongerThanSixHours = "longerThanSixHours",
    }
}

siri_enum! {
    /// Course classifications of a delay.
    DelaysType as "DelaysTypeEnum" {
        /// Delays on the road network as a result of any situation which causes hold-ups.
        Delays = "delays",
        /// Delays on the road network whose predicted duration cannot be estimated.
        DelaysOfUncertainDuration = "delaysOfUncertainDuration",
        /// Delays on the road network of unusual severity.
        LongDelays = "longDelays",
        /// Delays on the road network of abnormally unusual severity.
        VeryLongDelays = "veryLongDelays",
    }
}

siri_enum! {
    /// Values for TPEG Pts43 ServiceStatus, with additional values from Pti013
    ServiceCondition as "ServiceConditionEnumeration" {
        /// TPEG Pts43_0, unknown
        Unknown = "unknown",
        /// TPEG Pts43_1, delay
        Delay = "delay",
        /// TPEG Pts43_2, minor delays
        MinorDelays = "minorDelays",
        /// TPEG Pts43_3, major delays
        MajorDelays = "majorDelays",
        /// TPEG Pts43_4, operation time extension
        OperationTimeExtension = "operationTimeExtension",
        /// TPEG Pts43_5, on time
        OnTime = "onTime",
        /// TPEG Pts43_6, disturbance rectified
        DisturbanceRectified = "disturbanceRectified",
        /// TPEG Pts43_7, change of platform
        ChangeOfPlatform = "changeOfPlatform",
        /// TPEG Pts43_8, line cancellation
        LineCancellation = "lineCancellation",
        /// TPEG Pts43_9, trip cancellation
        TripCancellation = "tripCancellation",
        /// TPEG Pts43_10, boarding
        Boarding = "boarding",
        /// TPEG Pts43_11, go to gate
        GoToGate = "goToGate",
        /// TPEG Pts43_12, stop cancelled
        StopCancelled = "stopCancelled",
        /// TPEG Pts43_13, stop moved
        StopMoved = "stopMoved",
        /// TPEG Pts43_14, stop on demand
        StopOnDemand = "stopOnDemand",
        /// TPEG Pts43_15, additional stop
        AdditionalStop = "additionalStop",
        /// TPEG Pts43_16, substituted stop
        SubstitutedStop = "substitutedStop",
        /// TPEG Pts43_17, diverted
        Diverted = "diverted",
        /// TPEG Pts43_18, disruption
        Disruption = "disruption",
        /// TPEG Pts43_19, limited operation
        LimitedOperation = "limitedOperation",
        /// TPEG Pts43_20, discontinued operation
        DiscontinuedOperation = "discontinuedOperation",
        /// TPEG Pts43_21, irregular traffic
        IrregularTraffic = "irregularTraffic",
        /// TPEG Pts43_22, wagon order changed
        WagonOrderChanged = "wagonOrderChanged",
        /// TPEG Pts43_23, train shortened
        TrainShortened = "trainShortened",
        /// TPEG Pts43_24, additional ride
        AdditionalRide = "additionalRide",
        /// TPEG Pts43_25, replacement ride
        ReplacementRide = "replacementRide",
        /// TPEG Pts43_26, temporarily non-stopping
        TemporarilyNonStopping = "temporarilyNonStopping",
        /// TPEG Pts43_27, temporary stopplace
        TemporaryStopplace = "temporaryStopplace",
        /// TPEG Pts43_255, undefined status
        UndefinedStatus = "undefinedStatus",
        /// TPEG Pti13_1, DEPRECATED since SIRI 2.1
        Altered = "altered",
        /// TPEG Pti13_2, DEPRECATED since SIRI 2.1
        Cancelled = "cancelled",
        /// TPEG Pti13_3, DEPRECATED since SIRI 2.1
        Delayed = "delayed",
        /// TPEG Pti13_5, DEPRECATED since SIRI 2.1
        NoService = "noService",
        /// TPEG Pti13_6, DEPRECATED since SIRI 2.1
        Disrupted = "disrupted",
        /// TPEG Pti13_7, DEPRECATED since SIRI 2.1
        AdditionalService = "additionalService",
        /// TPEG Pti13_8, DEPRECATED since SIRI 2.1
        SpecialService = "specialService",
        /// TPEG Pti13_10, DEPRECATED since SIRI 2.1
        NormalService = "normalService",
        /// TPEG Pti13_11, DEPRECATED since SIRI 2.1
        IntermittentService = "intermittentService",
        /// TPEG Pti13_12, DEPRECATED since SIRI 2.1
        ShortFormedService = "shortFormedService",
        /// TPEG Pti13_13, DEPRECATED since SIRI 2.1
        FullLengthService = "fullLengthService",
        /// TPEG Pti13_14, DEPRECATED since SIRI 2.1
        ExtendedService = "extendedService",
        /// TPEG Pti13_15, DEPRECATED since SIRI 2.1
        SplittingTrain = "splittingTrain",
        /// TPEG Pti13_16, DEPRECATED since SIRI 2.1
        ReplacementTransport = "replacementTransport",
        /// TPEG Pti13_17, DEPRECATED since SIRI 2.1
        ArrivesEarly = "arrivesEarly",
        /// TPEG Pti13_18, DEPRECATED since SIRI 2.1
        ShuttleService = "shuttleService",
        /// TPEG Pti13_19, DEPRECATED since SIRI 2.1
        ReplacementService = "replacementService",
        /// TPEG Pti13_255, DEPRECATED since SIRI 2.1
        UndefinedServiceInformation = "undefinedServiceInformation",
    }
}

siri_enum! {
    /// Values for TPEG Pti025 - TicketRestrictionType
    TicketRestriction as "TicketRestrictionEnumeration" {
        /// TPEG Pti25_0, unknown
        Unknown = "unknown",
        /// TPEG Pti25_1, all ticket classes valid
        AllTicketClassesValid = "allTicketClassesValid",
        /// TPEG Pti25_2, full fare only
        FullFareOnly = "fullFareOnly",
        /// TPEG Pti25_3, certain tickets only
        CertainTicketsOnly = "certainTicketsOnly",
        /// TPEG Pti25_4, ticket with reservation
        TicketWithReservation = "ticketWithReservation",
        /// TPEG Pti25_5, special fare
        SpecialFare = "specialFare",
        /// TPEG Pti25_6, only tickets of specified operator
        OnlyTicketsOfSpecifiedOperator = "onlyTicketsOfSpecifiedOperator",
        /// TPEG Pti25_7, no restrictions
        NoRestrictions = "noRestrictions",
        /// TPEG Pti25_8, no off-peak tickets
        NoOffPeakTickets = "noOffPeakTickets",
        /// TPEG Pti25_9, no weekend return tickets
        NoWeekendReturnTickets = "noWeekendReturnTickets",
        /// TPEG Pti25_10, no reduced fare tickets
        NoReducedFareTickets = "noReducedFareTickets",
        /// TPEG Pti25_255, unknown ticket restriction
        UnknownTicketRestriction = "unknownTicketRestriction",
    }
}

siri_enum! {
    /// Enumeration of values for an accessibility value.
    Accessibility as "AccessibilityEnumeration" {
        /// unknown
        Unknown = "unknown",
        /// false
        False = "false",
        /// true
        True = "true",
    }
}

siri_enum! {
    /// Values for AccessibilityFeature - TPEG Pts040 and IFOPT
    AccessibilityFeature as "AccessibilityFeatureEnumeration" {
        /// IFOPT, TPEG Pts40_0, unknown
        Unknown = "unknown",
        /// TPEG Pts40_1, single step
        SingleStep = "singleStep",
        /// IFOPT, TPEG Pts40_2, stairs
        Stairs = "stairs",
        /// IFOPT, TPEG Pts40_3, escalator
        Escalator = "escalator",
        /// IFOPT, TPEG Pts40_4, travelator / moving walkway
        Travelator = "travelator",
        /// IFOPT, TPEG Pts40_5, lift / elevator
        Lift = "lift",
        /// IFOPT, TPEG Pts40_6, ramp
        Ramp = "ramp",
        /// TPEG Pts40_7, mind the gap
        MindTheGap = "mindTheGap",
        /// TPEG Pts40_8, tactile paving
        TactilePaving = "tactilePaving",
        /// IFOPT, series of stairs
        SeriesOfStairs = "seriesOfStairs",
        /// IFOPT, shuttle
        Shuttle = "shuttle",
        /// IFOPT, barrier
        Barrier = "barrier",
        /// IFOPT, narrow entrance
        NarrowEntrance = "narrowEntrance",
        /// IFOPT, confined space
        ConfinedSpace = "confinedSpace",
        /// IFOPT, queue management
        QueueManagement = "queueManagement",
        /// IFOPT, none
        None = "none",
        /// IFOPT, other
        Other = "other",
        /// TPEG Pts40_255, undefined accessibility feature type
        Undefined = "undefined",
    }
}

siri_enum! {
    /// Types of areas of interest.
    AreaOfInterest as "AreaOfInterestEnum" {
        /// Area of the whole European continent.
        ContinentWide = "continentWide",
        /// Whole area of the specific country.
        National = "national",
        /// Area of countries which are neighbouring the one specified.
        NeighbouringCountries = "neighbouringCountries",
        /// Non specified area.
        NotSpecified = "notSpecified",
        /// Area of the local region.
        Regional = "regional",
    }
}

siri_enum! {
    /// Classification of the timeliness of the CALL, according to a fixed list of values. This may reflect a presentation policy, for example CALLs less than one minute behind target time are still classified as on-time. Applications may use this to guide their own presentation of times.
    CallStatus as "CallStatusEnumeration" {
        /// Service is on time.
        OnTime = "onTime",
        /// Service is earlier than expected.
        Early = "early",
        /// Service is delayed.
        Delayed = "delayed",
        /// Service is cancelled.
        Cancelled = "cancelled",
        /// Service has arrived.
        Arrived = "arrived",
        /// departed
        Departed = "departed",
        /// missed
        Missed = "missed",
        /// There is no information about the service.
        NoReport = "noReport",
        /// Service is not expected to call this stop. For instance a flexible service that has not yet been preordered.
        NotExpected = "notExpected",
    }
}

siri_enum! {
    /// Values for DIRECTION of CONNECTION link or SERVCIE JOURNEY INTERCHANGE.
    ConnectionDirection as "ConnectionDirectionEnumeration" {
        /// to
        To = "to",
        /// from
        From = "from",
        /// both
        Both = "both",
    }
}

siri_enum! {
    /// Allowed values for the status of a MONITORED FACILITY.
    FacilityStatus as "FacilityStatusEnumeration" {
        /// unknown
        Unknown = "unknown",
        /// available
        Available = "available",
        /// notAvailable
        NotAvailable = "notAvailable",
        /// partiallyAvailable
        PartiallyAvailable = "partiallyAvailable",
        /// added
        Added = "added",
        /// removed
        Removed = "removed",
    }
}

siri_enum! {
    /// Values for TPEG Pti31 - InterchangeStatus
    InterchangeStatus as "InterchangeStatusEnumeration" {
        /// TPEG Pti31_0, unknown
        Unknown = "unknown",
        /// TPEG Pti31_1, connection
        Connection = "connection",
        /// TPEG Pti31_2, replacement
        Replacement = "replacement",
        /// TPEG Pti31_3, alternative
        Alternative = "alternative",
        /// TPEG Pti31_4, connection not held
        ConnectionNotHeld = "connectionNotHeld",
        /// TPEG Pti31_5, connection held
        ConnectionHeld = "connectionHeld",
        /// TPEG Pti31_6, status of connection undecided
        StatusOfConnectionUndecided = "statusOfConnectionUndecided",
        /// TPEG Pti31_255, undefined cross reference information
        UndefinedCrossReferenceInformation = "undefinedCrossReferenceInformation",
        /// Interchange is planned but was updated as a result of changes in the QUAYs or arrival/departure times. Can be used if the status is a combination of the other values. (since SIRI 2.1)
        ConnectionChanged = "connectionChanged",
        /// An important function of connection protection is the ability to hold back a distributor VEHICLE (i.e. prolonged waiting) to allow passengers to transfer from delayed feeders. To achieve this a distributorWaitProlonged status shall be communicated back to the feeder VEHICLEs to inform the passengers about the new departure time of the distributor or even a willWait guarantee. (since SIRI 2.1)
        DistributorWaitProlonged = "distributorWaitProlonged",
        /// Used to provide the passengers with information about a new departure platform of the distributor VEHICLE if the distributor changes its planned stopping position. (since SIRI 2.1)
        DeparturePlatformChanged = "departurePlatformChanged",
        /// Interchange is an addition to the plan. (since SIRI 2.1)
        ExtraInterchange = "extraInterchange",
        /// Interchange is a cancellation of an interchange in the plan. (since SIRI 2.1)
        Cancelled = "cancelled",
        /// Loss of the inbound connection indicates the cancellation of the visit (of the FeederJourney) to the FeederArrivalStop, or a severely delayed arrival. This can lead to the distributor VEHICLE abandoning the connection. Reasons for the loss of a feeder include (but are not limited to) the cancellation of the feeder VEHICLE, diversion/rerouting of the feeder VEHICLE, disruption of a line section or journey part of the feeder VEHICLE etc. (since SIRI 2.1)
        FeederArrivalCancellation = "feederArrivalCancellation",
        /// Indicates the loss of an outbound connection, i.e., is used to signal the cancellation of the onward connection to the passengers in the feeder VEHICLEs. (since SIRI 2.1)
        DistributorDepartureCancellation = "distributorDepartureCancellation",
        /// DEPRECATED since SIRI 2.1 - use statusOfConnectionUndecided instead
        StatusOfConenctionUndecided = "statusOfConenctionUndecided",
    }
}

siri_enum! {
    /// Values for ROUTE POINT type that correspond to TPEG Pts44: StopPlaceUsage (Pti15: deprecated since SIRI 2.1).
    RoutePointType as "RoutePointTypeEnumeration" {
        /// TPEG Pti15_0, Pts44_0, unknown
        Unknown = "unknown",
        /// TPEG Pts44_1, origin
        Origin = "origin",
        /// TPEG Pti15_2, Pts44_2, destination
        Destination = "destination",
        /// TPEG Pts44_3, intermediate.
        Intermediate = "intermediate",
        /// TPEG Pts44_4, leg board
        LegBoard = "legBoard",
        /// TPEG Pts44_5, leg intermediate
        LegIntermediate = "legIntermediate",
        /// TPEG Pts44_6, leg alight
        LegAlight = "legAlight",
        /// TPEG Pts44_7, first route point
        FirstRoutePoint = "firstRoutePoint",
        /// TPEG Pts44_8, last route point
        LastRoutePoint = "lastRoutePoint",
        /// TPEG Pts44_9, affected STOP PLACE
        AffectedStopplace = "affectedStopplace",
        /// TPEG Pts44_10, presented STOP PLACE
        PresentedStopplace = "presentedStopplace",
        /// TPEG Pts44_255, undefined STOP PLACE usage
        UndefinedStopplaceUsage = "undefinedStopplaceUsage",
        /// DEPRECATED since SIRI 2.1 and replaced by Pts44_1 value 'origin' (TPEG Pti15_1 - start point) .
        StartPoint = "startPoint",
        /// DEPRECATED since SIRI 2.1 (TPEG Pti15_3 - stop)
        Stop = "stop",
        /// DEPRECATED since SIRI 2.1 (TPEG Pti15_4 - via)
        Via = "via",
        /// DEPRECATED since SIRI 2.1 (TPEG Pti15_5 - not-stopping)
        NotStopping = "notStopping",
        /// DEPRECATED since SIRI 2.1 (TPEG Pti15_6 - temporary stop)
        TemporaryStop = "temporaryStop",
        /// DEPRECATED since SIRI 2.1 (TPEG Pti15_7 - temporarily not-stopping)
        TemporarilyNotStopping = "temporarilyNotStopping",
        /// DEPRECATED since SIRI 2.1 (TPEG Pti15_8 - exceptional stop)
        ExceptionalStop = "exceptionalStop",
        /// DEPRECATED since SIRI 2.1 (TPEG Pti15_9 - additional stop)
        AdditionalStop = "additionalStop",
        /// DEPRECATED since SIRI 2.1 (TPEG Pti15_10 - request stop)
        RequestStop = "requestStop",
        /// DEPRECATED since SIRI 2.1 (TPEG Pti15_11 - front train destination)
        FrontTrainDestination = "frontTrainDestination",
        /// DEPRECATED since SIRI 2.1 (TPEG Pti15_12 - rear train destination)
        RearTrainDestination = "rearTrainDestination",
        /// DEPRECATED since SIRI 2.1 (TPEG Pti15_13 - through service destination)
        ThroughServiceDestination = "throughServiceDestination",
        /// DEPRECATED since SIRI 2.1 (TPEG Pti15_14 - not via)
        NotVia = "notVia",
        /// DEPRECATED since SIRI 2.1 (TPEG Pti15_15 - altered start point)
        AlteredStartPoint = "alteredStartPoint",
        /// DEPRECATED since SIRI 2.1 (TPEG Pti15_16 - altered destination)
        AlteredDestination = "alteredDestination",
        /// DEPRECATED since SIRI 2.1 (TPEG Pti15_255 - undefined route point)
        UndefinedRoutePoint = "undefinedRoutePoint",
    }
}

siri_enum! {
    /// Enumeration of SITE COMPONENT Types.
    StopPlaceComponentType as "StopPlaceComponentTypeEnumeration" {
        /// quay
        Quay = "quay",
        /// accessSpace
        AccessSpace = "accessSpace",
        /// entrance
        Entrance = "entrance",
        /// boardingPosition
        BoardingPosition = "boardingPosition",
        /// stoppingPlace
        StoppingPlace = "stoppingPlace",
    }
}

siri_enum! {
    /// Values for STOP PLACE types - TPEG Pts041 and IFOPT
    StopPlaceType as "StopPlaceTypeEnumeration" {
        /// TPEG Pts41_0, unknown
        Unknown = "unknown",
        /// TPEG Pts41_1, railway station
        RailwayStation = "railwayStation",
        /// TPEG Pts41_2, underground station
        UndergroundStation = "undergroundStation",
        /// IFOPT, TPEG Pts41_3, tram station
        TramStation = "tramStation",
        /// IFOPT, TPEG Pts41_4, bus station
        BusStation = "busStation",
        /// IFOPT, TPEG Pts41_5, airport
        Airport = "airport",
        /// TPEG Pts41_6, pier
        Pier = "pier",
        /// IFOPT, TPEG Pts41_7, harbour port
        HarbourPort = "harbourPort",
        /// ,IFOPT, TPEG Pts41_8, ferry stop
        FerryStop = "ferryStop",
        /// TPEG Pts41_9, light railway station
        LightRailwayStation = "lightRailwayStation",
        /// TPEG Pts41_10, cogwheel station
        CogwheelStation = "cogwheelStation",
        /// TPEG Pts41_11, funicular station
        FunicularStation = "funicularStation",
        /// TPEG Pts41_12, ropeway station
        RopewayStation = "ropewayStation",
        /// IFOPT, coach station
        CoachStation = "coachStation",
        /// IFOPT, ferry port
        FerryPort = "ferryPort",
        /// IFOPT, on-street bus stop
        OnStreetBus = "onStreetBus",
        /// IFOPT, on-street tram stop
        OnStreetTram = "onStreetTram",
        /// IFOPT, ski lift
        SkiLift = "skiLift",
        /// IFOPT, other
        Other = "other",
        /// TPEG Pts41_255, undefined STOP PLACE type
        UndefinedStopPlaceType = "undefinedStopPlaceType",
        /// IFOPT, deprecated (SIRI 2.1), use railwayStation
        RailStation = "railStation",
        /// IFOPT, deprecated (SIRI 2.1), use undergroundStation
        MetroStation = "metroStation",
    }
}

siri_enum! {
    /// Values for TPEG Pts017 - ServiceDeliveryPointType
    StopPointType as "StopPointTypeEnumeration" {
        /// TPEG Pts17_0, unknown
        Unknown = "unknown",
        /// TPEG Pts17_1, platform number
        PlatformNumber = "platformNumber",
        /// TPEG Pts17_2, terminal gate
        TerminalGate = "terminalGate",
        /// TPEG Pts17_3, ferry berth
        FerryBerth = "ferryBerth",
        /// TPEG Pts17_4, harbour pier
        HarbourPier = "harbourPier",
        /// TPEG Pts17_5, unknown
        LandingStage = "landingStage",
        /// TPEG Pts17_6, bus stop
        BusStop = "busStop",
        /// TPEG Pts17_255, undefined service delivery point type
        UndefinedStopPointType = "undefinedStopPointType",
        /// DEPRECATED since SIRI 2.1 - use undefinedStopPointType
        UndefinedBookingInformation = "undefinedBookingInformation",
    }
}


siri_enum! {
    /// Passenger load status of a VEHICLE - GTFS-R / TPEG Pts045
    Occupancy as "OccupancyEnumeration" {
        /// TPEG Pts45_0, unknown
        Unknown = "unknown",
        /// GTFS-R "EMPTY". The vehicle is considered empty by most measures, and
        /// has few or no passengers onboard, but is still accepting passengers.
        Empty = "empty",
        /// GTFS-R "MANY_SEATS_AVAILABLE" / TPEG Pts45_1, many seats available.
        ManySeatsAvailable = "manySeatsAvailable",
        /// GTFS-R "FEW_SEATS_AVAILABLE" / TPEG Pts45_2, few seats available.
        FewSeatsAvailable = "fewSeatsAvailable",
        /// GTFS-R "STANDING_ROOM_ONLY" / TPEG Pts45_4, standing room only.
        StandingRoomOnly = "standingRoomOnly",
        /// GTFS-R "CRUSHED_STANDING_ROOM_ONLY". The vehicle can currently
        /// accommodate only standing passengers and has limited space for them.
        CrushedStandingRoomOnly = "crushedStandingRoomOnly",
        /// GTFS-R "FULL" / TPEG Pts45_5, full
        Full = "full",
        /// GTFS-R "NOT_ACCEPTING_PASSENGERS". The vehicle cannot accept passengers.
        NotAcceptingPassengers = "notAcceptingPassengers",
        /// TPEG Pts45_255, undefined occupancy
        Undefined = "undefined",
        /// DEPRECATED since SIRI 2.1 - use a more specific value
        SeatsAvailable = "seatsAvailable",
        /// DEPRECATED since SIRI 2.1 - use a more specific value
        StandingAvailable = "standingAvailable",
    }
}

siri_enum! {
    /// Classification of the rate of progress of VEHICLE according a fixed list of values.
    ProgressRate as "ProgressRateEnumeration" {
        /// Vehicle is stationary.
        NoProgress = "noProgress",
        /// Vehicle is proceeding slower than normal.
        SlowProgress = "slowProgress",
        /// Vehicle is proceeding at a normal rate.
        NormalProgress = "normalProgress",
        /// Vehicle is proceeding faster than normal.
        FastProgress = "fastProgress",
        /// There is no data.
        Unknown = "unknown",
    }
}

siri_enum! {
    /// Classification of the State of the VEHICLE JOURNEY according to a fixed list
    /// of values.
    VehicleStatus as "VehicleStatusEnumeration" {
        /// Service is expected to be performed.
        Expected = "expected",
        /// Service is not expected to be run. For instance a flexible service that
        /// has not yet been preordered.
        NotExpected = "notExpected",
        /// cancelled
        Cancelled = "cancelled",
        /// assigned
        Assigned = "assigned",
        /// signedOn
        SignedOn = "signedOn",
        /// atOrigin
        AtOrigin = "atOrigin",
        /// Service has departed from first stop.
        InProgress = "inProgress",
        /// aborted
        Aborted = "aborted",
        /// offRoute
        OffRoute = "offRoute",
        /// It has been detected that the Service was completed.
        Completed = "completed",
        /// It is assumed that the Service has completed.
        AssumedCompleted = "assumedCompleted",
        /// notRun
        NotRun = "notRun",
    }
}

siri_enum! {
    /// Allowed types activity for FirstOrLastJourney.
    FirstOrLastJourney as "FirstOrLastJourneyEnumeration" {
        /// firstServiceOfDay
        FirstServiceOfDay = "firstServiceOfDay",
        /// otherService
        OtherService = "otherService",
        /// lastServiceOfDay
        LastServiceOfDay = "lastServiceOfDay",
        /// unspecified
        Unspecified = "unspecified",
    }
}

siri_enum! {
    /// Possible reasons for a change in prediction (in)accuracy.
    PredictionInaccurateReason as "PredictionInaccurateReasonEnumeration" {
        /// Prediction is inaccurate because of a traffic jam.
        VehicleInTrafficJam = "vehicleInTrafficJam",
        /// Prediction is inaccurate because of technical problems.
        TechnicalProblem = "technicalProblem",
        /// Prediction is inaccurate because of a despatching alteration.
        DispatchAction = "dispatchAction",
        /// Prediction is inaccurate because communication errors have prevented
        /// any updates.
        MissingUpdate = "missingUpdate",
        /// Prediction is inaccurate but the reason is unknown.
        Unknown = "unknown",
    }
}

siri_enum! {
    /// Allowed types of relation between JOURNEYs.
    JourneyRelationType as "JourneyRelationTypeEnumeration" {
        /// The journey is a continuation of the specified RelatedJourney at the
        /// stop point given in CallInfo. Passengers don't need to change vehicles.
        ContinuationOfJourney = "ContinuationOfJourney",
        /// The journey is continued by the specified RelatedJourney at the stop
        /// point given in CallInfo. Passengers don't need to change vehicles.
        ContinuedByJourney = "ContinuedByJourney",
        /// The journey splits into multiple RelatedJourneys at the stop point given
        /// in CallInfo.
        SplitsIntoJourneys = "SplitsIntoJourneys",
        /// The journey is a continuation of a single RelatedJourney splitting into
        /// multiple journeys at the stop point given in CallInfo.
        ContinuationOfSplitJourney = "ContinuationOfSplitJourney",
        /// The journey is the continuation of multiple RelatedJourneys joining
        /// together at the stop point given in CallInfo.
        JoiningOfJourneys = "JoiningOfJourneys",
        /// The journey is continued by a single RelatedJourney after joining other
        /// journeys at the stop point given in CallInfo.
        ContinuedByJoinedJourney = "ContinuedByJoinedJourney",
        /// The journey replaces one or more partially or fully cancelled
        /// RelatedJourneys during the journey part named in JourneyPartInfo.
        ReplacementOfJourney = "ReplacementOfJourney",
        /// The partially or fully cancelled journey is replaced by one or more
        /// RelatedJourneys during the journey part named in JourneyPartInfo.
        ReplacedByJourney = "ReplacedByJourney",
        /// The journey partially or fully supports one or more RelatedJourneys
        /// during the journey part named in JourneyPartInfo.
        SupportOfJourney = "SupportOfJourney",
        /// The journey is partially or fully supported by one or more
        /// RelatedJourneys during the journey part named in JourneyPartInfo.
        SupportedByJourney = "SupportedByJourney",
    }
}

siri_enum! {
    /// Characterisation of nested QUAYs as part of a STOP ASSIGNMENT. (since SIRI 2.1)
    TypeOfNestedQuay as "TypeOfNestedQuayEnumeration" {
        /// A type of QUAY that consists of multiple QUAYs of type `platform`, e.g.
        /// the lower and upper level of a station.
        PlatformGroup = "platformGroup",
        /// A type of QUAY that consists of at least two child QUAYs of type
        /// `platformEdge`.
        Platform = "platform",
        /// A type of QUAY which allows direct access to a VEHICLE, e.g. an
        /// on-street bus stop, or consists of child QUAYs of type `platformSector`.
        PlatformEdge = "platformEdge",
        /// A part of a `platformEdge`, e.g. "A", "B", "C", helping passengers find
        /// a specific part of a vehicle.
        PlatformSector = "platformSector",
    }
}

siri_enum! {
    /// Allowed values for TYPE OF TRAIN ELEMENT. (since SIRI 2.1)
    TrainElementType as "TrainElementTypeEnumeration" {
        /// buffetCar
        BuffetCar = "buffetCar",
        /// carriage
        Carriage = "carriage",
        /// engine
        Engine = "engine",
        /// carTransporter
        CarTransporter = "carTransporter",
        /// sleeperCarriage
        SleeperCarriage = "sleeperCarriage",
        /// luggageVan
        LuggageVan = "luggageVan",
        /// restaurantCarriage
        RestaurantCarriage = "restaurantCarriage",
        /// other
        Other = "other",
    }
}

siri_enum! {
    /// Allowed values for TRAIN SIZE. (since SIRI 2.1)
    TrainSize as "TrainSizeEnumeration" {
        /// normal
        Normal = "normal",
        /// short
        Short = "short",
        /// long
        Long = "long",
    }
}

siri_enum! {
    /// Allowed values for TYPE OF FUEL. (since SIRI 2.1)
    TypeOfFuel as "TypeOfFuelEnumeration" {
        /// petrol
        Petrol = "petrol",
        /// diesel
        Diesel = "diesel",
        /// naturalGas
        NaturalGas = "naturalGas",
        /// biodiesel
        Biodiesel = "biodiesel",
        /// electricity
        Electricity = "electricity",
        /// hydrogen
        Hydrogen = "hydrogen",
        /// other
        Other = "other",
        /// unknown
        Unknown = "unknown",
    }
}

siri_enum! {
    /// Values for Fare Class Facility. (since SIRI 2.1)
    ///
    /// The published schema lists `secondClass` twice, once with a trailing space
    /// left over from SIRI 2.0; both spell the same value, so there is one variant.
    FareClass as "FareClassEnumeration" {
        /// pti23_0
        Unknown = "unknown",
        /// pti23_6
        FirstClass = "firstClass",
        /// pti23_7
        SecondClass = "secondClass",
        /// pti23_8
        ThirdClass = "thirdClass",
        /// preferente
        Preferente = "preferente",
        /// pti23_6_1
        PremiumClass = "premiumClass",
        /// Business Class - pti23_10
        BusinessClass = "businessClass",
        /// Standard class - pti23_7
        StandardClass = "standardClass",
        /// turista
        Turista = "turista",
        /// pti23_9
        EconomyClass = "economyClass",
        /// any
        Any = "any",
    }
}

siri_enum! {
    /// Allowed values for VEHICLE IN FORMATION STATUS CODE. (since SIRI 2.1)
    VehicleInFormationStatus as "VehicleInFormationStatusEnumeration" {
        /// unknown
        Unknown = "unknown",
        /// available
        Available = "available",
        /// notAvailable
        NotAvailable = "notAvailable",
        /// partiallyAvailable
        PartiallyAvailable = "partiallyAvailable",
        /// added
        Added = "added",
        /// removed
        Removed = "removed",
        /// defective
        Defective = "defective",
        /// closed
        Closed = "closed",
        /// booked
        Booked = "booked",
        /// noRestaurantService
        NoRestaurantService = "noRestaurantService",
        /// open
        Open = "open",
    }
}

siri_enum! {
    /// Allowed values for FORMATION CHANGE CODE. (since SIRI 2.1)
    FormationChange as "FormationChangeEnumeration" {
        /// changedFormation
        ChangedFormation = "changedFormation",
        /// reversedFormation
        ReversedFormation = "reversedFormation",
        /// missingVehicles
        MissingVehicles = "missingVehicles",
        /// extraVehicles
        ExtraVehicles = "extraVehicles",
        /// missingTrainInCompoundTrain
        MissingTrainInCompoundTrain = "missingTrainInCompoundTrain",
        /// extraTrainInCompoundTrain
        ExtraTrainInCompoundTrain = "extraTrainInCompoundTrain",
        /// missingFamilyCoach
        MissingFamilyCoach = "missingFamilyCoach",
        /// missingThroughCoach
        MissingThroughCoach = "missingThroughCoach",
        /// missingLowFloorCoach
        MissingLowFloorCoach = "missingLowFloorCoach",
        /// missingRestaurantCoach
        MissingRestaurantCoach = "missingRestaurantCoach",
        /// missingWheelchairSpaces
        MissingWheelchairSpaces = "missingWheelchairSpaces",
    }
}

siri_enum! {
    /// Detail Levels for Estimated Timetable Request.
    EstimatedTimetableDetail as "EstimatedTimetableDetailEnumeration" {
        /// Return only the minimum amount of optional data for each stop visit to
        /// provide a display: a time at stop, LINE name and destination name.
        Minimum = "minimum",
        /// Return minimum and other available basic details for each stop visit.
        /// Do not include data on times at next stop or destination.
        Basic = "basic",
        /// Return all basic data, and also origin VIA points and destination.
        Normal = "normal",
        /// Return, in addition to normal data, the estimated call data.
        Calls = "calls",
        /// Return all available data for each journey, including calls.
        Full = "full",
    }
}

siri_enum! {
    /// Detail Levels for Vehicle Monitoring Request.
    VehicleMonitoringDetail as "VehicleMonitoringDetailEnumeration" {
        /// Return only the minimum amount of optional data for each stop event to
        /// provide a display: a time, line name and destination name.
        Minimum = "minimum",
        /// Return minimum and other available basic details for each stop event.
        /// Do not include data on time at next stop or destination.
        Basic = "basic",
        /// Return all basic data, and also arrival times at DESTINATION.
        Normal = "normal",
        /// Return all available data for each stop event, including previous and
        /// onward CALLs with passing times for the JOURNEY PATTERN.
        Calls = "calls",
    }
}
