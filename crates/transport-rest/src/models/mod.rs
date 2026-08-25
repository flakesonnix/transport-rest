//! Strongly typed models of the transport.rest wire format (FPTF).
//!
//! All structs tolerate unknown fields (ignored on deserialize) and optional
//! data (`null` or absent). Open enums capture unknown values in an
//! `Other` variant so API evolution never breaks deserialization.

pub mod enums;
pub mod journey;
pub mod place;
pub mod polyline;
pub mod response;
pub(crate) mod transit;

pub use enums::{DbProfile, Mode, PrognosisType, RemarkKind};
pub use journey::{Departure, Frame, Journey, Leg, Movement, ReachableDuration, Stopover, Trip};
pub use place::{Location, LocationResult, Place, Station, Stop, StopOrStation};
pub use polyline::{GeometryPoint, Polyline, PolylineFeature};
pub use response::{
    Aliases, ArrivalsResponse, DeparturesResponse, JourneyResponse, JourneysResponse,
    LocationsResponse, RadarResponse, ReachableFromResponse, StationsResponse, TripResponse,
    TripsResponse,
};
pub use transit::{Cycle, Line, Operator, Price, Remark};
