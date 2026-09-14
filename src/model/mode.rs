//! Modes of transport.
//!
//! SIRI states a mode in two levels: a coarse [`VehicleModesOfTransport`] and,
//! optionally, one submode drawn from the list belonging to that mode. The schema
//! writes the submode as its own element — `<BusSubmode>`, `<RailSubmode>` and so
//! on — so a structure carrying a mode has one optional field per submode family
//! and at most one of them is set. [`Submode`] reports which.

use crate::enumerations::{
    AirSubmodesOfTransport, BusSubmodesOfTransport, CoachSubmodesOfTransport,
    MetroSubmodesOfTransport, RailSubmodesOfTransport, TelecabinSubmodesOfTransport,
    TramSubmodesOfTransport, WaterSubmodesOfTransport,
};

/// The submode a structure carries, if any.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Submode {
    /// A kind of air service.
    Air(AirSubmodesOfTransport),
    /// A kind of bus service.
    Bus(BusSubmodesOfTransport),
    /// A kind of coach service.
    Coach(CoachSubmodesOfTransport),
    /// A kind of metro service.
    Metro(MetroSubmodesOfTransport),
    /// A kind of rail service.
    Rail(RailSubmodesOfTransport),
    /// A kind of tram service.
    Tram(TramSubmodesOfTransport),
    /// A kind of water-borne service.
    Water(WaterSubmodesOfTransport),
    /// A kind of cable-drawn service.
    Telecabin(TelecabinSubmodesOfTransport),
}

impl Submode {
    /// The token this submode is written as on the wire.
    pub fn as_str(&self) -> &str {
        match self {
            Submode::Air(value) => value.as_str(),
            Submode::Bus(value) => value.as_str(),
            Submode::Coach(value) => value.as_str(),
            Submode::Metro(value) => value.as_str(),
            Submode::Rail(value) => value.as_str(),
            Submode::Tram(value) => value.as_str(),
            Submode::Water(value) => value.as_str(),
            Submode::Telecabin(value) => value.as_str(),
        }
    }
}

impl core::fmt::Display for Submode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}
