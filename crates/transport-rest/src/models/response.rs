//! Response envelopes and shared type aliases.

use serde::{Deserialize, Serialize};

use super::journey::{Departure, Journey, Movement, ReachableDuration, Trip};
use super::place::LocationResult;

/// Shared map types used across models.
pub mod Aliases {
    /// Profile-specific product flags keyed by product name
    /// (e.g. `"bus": true`). Which keys exist depends on the provider.
    pub type Products = std::collections::BTreeMap<String, bool>;

    /// External identifiers of a stop/station (e.g. `dhid`).
    pub type Ids = std::collections::BTreeMap<String, String>;

    /// ISO date -> served? db-vendo exposes this as `serviceDays`,
    /// hafas-client style responses as `scheduledDays`.
    pub type ServiceDays = std::collections::BTreeMap<String, bool>;

    /// Weekday abbreviation -> opening hours string.
    pub type OpeningHours = std::collections::BTreeMap<String, String>;
}

/// Result list of location search & nearby queries.
pub type LocationsResponse = Vec<LocationResult>;

/// Static station search result list (DB `/stations`, BVG/VBB `/stops`
/// return compatible objects).
pub type StationsResponse = Vec<StationEntry>;

/// A single entry of a static station dataset; modeled loosely because the
/// datasets differ per instance.
pub type StationEntry = LocationResult;

/// Response of departure board queries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeparturesResponse {
    /// Departures ordered by time.
    #[serde(default)]
    pub departures: Vec<Departure>,
    /// When the realtime data was last updated (unix epoch seconds), if known.
    #[serde(rename = "realtimeDataUpdatedAt", default, with = "crate::util::lenient_i64", skip_serializing_if = "Option::is_none")]
    pub realtime_data_updated_at: Option<i64>,
}

/// Response of arrival board queries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ArrivalsResponse {
    /// Arrivals ordered by time.
    #[serde(default)]
    pub arrivals: Vec<Departure>,
    /// When the realtime data was last updated (unix epoch seconds), if known.
    #[serde(rename = "realtimeDataUpdatedAt", default, with = "crate::util::lenient_i64", skip_serializing_if = "Option::is_none")]
    pub realtime_data_updated_at: Option<i64>,
}

/// Response of journey searches.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct JourneysResponse {
    /// Matching journeys, ordered by time.
    #[serde(default)]
    pub journeys: Vec<Journey>,
    /// Pass as `earlier_than` to fetch earlier journeys.
    #[serde(rename = "earlierRef", default, skip_serializing_if = "Option::is_none")]
    pub earlier_ref: Option<String>,
    /// Pass as `later_than` to fetch later journeys.
    #[serde(rename = "laterRef", default, skip_serializing_if = "Option::is_none")]
    pub later_ref: Option<String>,
    /// When the realtime data was last updated (unix epoch seconds), if known.
    #[serde(rename = "realtimeDataUpdatedAt", default, with = "crate::util::lenient_i64", skip_serializing_if = "Option::is_none")]
    pub realtime_data_updated_at: Option<i64>,
}

/// Response of journey refreshes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JourneyResponse {
    /// The refreshed journey.
    pub journey: Journey,
    /// When the realtime data was last updated (unix epoch seconds), if known.
    #[serde(rename = "realtimeDataUpdatedAt", default, with = "crate::util::lenient_i64", skip_serializing_if = "Option::is_none")]
    pub realtime_data_updated_at: Option<i64>,
}

/// Response of trip lookups.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TripResponse {
    /// The requested trip.
    pub trip: Trip,
    /// When the realtime data was last updated (unix epoch seconds), if known.
    #[serde(rename = "realtimeDataUpdatedAt", default, with = "crate::util::lenient_i64", skip_serializing_if = "Option::is_none")]
    pub realtime_data_updated_at: Option<i64>,
}

/// Response of trips-by-name queries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TripsResponse {
    /// Matching trips.
    #[serde(default)]
    pub trips: Vec<Trip>,
    /// When the realtime data was last updated (unix epoch seconds), if known.
    #[serde(rename = "realtimeDataUpdatedAt", default, with = "crate::util::lenient_i64", skip_serializing_if = "Option::is_none")]
    pub realtime_data_updated_at: Option<i64>,
}

/// Response of radar queries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RadarResponse {
    /// Vehicles moving inside the queried bounding box.
    #[serde(default)]
    pub movements: Vec<Movement>,
    /// When the realtime data was last updated (unix epoch seconds), if known.
    #[serde(rename = "realtimeDataUpdatedAt", default, with = "crate::util::lenient_i64", skip_serializing_if = "Option::is_none")]
    pub realtime_data_updated_at: Option<i64>,
}

/// Response of reachable-from queries.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReachableFromResponse {
    /// Reachable durations & stations.
    #[serde(default)]
    pub reachable: Vec<ReachableDuration>,
    /// When the realtime data was last updated (unix epoch seconds), if known.
    #[serde(rename = "realtimeDataUpdatedAt", default, with = "crate::util::lenient_i64", skip_serializing_if = "Option::is_none")]
    pub realtime_data_updated_at: Option<i64>,
}
