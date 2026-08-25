//! Journeys, trips and their building blocks.

use serde::{Deserialize, Serialize};

use super::enums::PrognosisType;
use super::place::{Place, StopOrStation};
use super::polyline::Polyline;
use super::transit::{Cycle, Line, Operator, Price, Remark};
use super::Aliases;

/// A vehicle stopping at a stop at specific times.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Stopover {
    /// The stop/station.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<StopOrStation>,
    /// Realtime arrival; `None` on the first stopover or if cancelled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arrival: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Scheduled arrival.
    #[serde(
        rename = "plannedArrival",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub planned_arrival: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Prognosed arrival.
    #[serde(
        rename = "prognosedArrival",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prognosed_arrival: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Arrival delay in seconds.
    #[serde(
        rename = "arrivalDelay",
        default,
        with = "crate::util::lenient_i64",
        skip_serializing_if = "Option::is_none"
    )]
    pub arrival_delay: Option<i64>,
    /// Realtime arrival platform.
    #[serde(
        rename = "arrivalPlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub arrival_platform: Option<String>,
    /// Scheduled arrival platform.
    #[serde(
        rename = "plannedArrivalPlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub planned_arrival_platform: Option<String>,
    /// Prognosed arrival platform.
    #[serde(
        rename = "prognosedArrivalPlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prognosed_arrival_platform: Option<String>,
    /// Arrival prognosis class.
    #[serde(
        rename = "arrivalPrognosisType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub arrival_prognosis_type: Option<PrognosisType>,
    /// Realtime departure; `None` on the last stopover or if cancelled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub departure: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Scheduled departure.
    #[serde(
        rename = "plannedDeparture",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub planned_departure: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Prognosed departure.
    #[serde(
        rename = "prognosedDeparture",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prognosed_departure: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Departure delay in seconds.
    #[serde(
        rename = "departureDelay",
        default,
        with = "crate::util::lenient_i64",
        skip_serializing_if = "Option::is_none"
    )]
    pub departure_delay: Option<i64>,
    /// Realtime departure platform.
    #[serde(
        rename = "departurePlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub departure_platform: Option<String>,
    /// Scheduled departure platform.
    #[serde(
        rename = "plannedDeparturePlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub planned_departure_platform: Option<String>,
    /// Prognosed departure platform.
    #[serde(
        rename = "prognosedDeparturePlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prognosed_departure_platform: Option<String>,
    /// Departure prognosis class.
    #[serde(
        rename = "departurePrognosisType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub departure_prognosis_type: Option<PrognosisType>,
    /// Remarks/hints/warnings for this stopover.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remarks: Vec<Remark>,
    /// Vehicle passes without stopping?
    #[serde(rename = "passBy", default, skip_serializing_if = "Option::is_none")]
    pub pass_by: Option<bool>,
    /// Stop is cancelled?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancelled: Option<bool>,
    /// Extra, unscheduled stop?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional: Option<bool>,
}

/// One leg of a journey.
///
/// Walking/transfer legs carry `walking`/`transfer = true` and no [`Leg::line`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Leg {
    /// ID of the underlying trip, if this is a scheduled leg.
    #[serde(rename = "tripId", default, skip_serializing_if = "Option::is_none")]
    pub trip_id: Option<String>,
    /// Where the leg starts.
    pub origin: Place,
    /// Where the leg ends.
    pub destination: Place,
    /// Line of the vehicle; absent for walking/transfer legs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<Line>,
    /// Head-sign / terminus of the vehicle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    /// Operating company.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<Operator>,
    /// Realtime departure time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub departure: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Scheduled departure time.
    #[serde(
        rename = "plannedDeparture",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub planned_departure: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Prognosed departure time.
    #[serde(
        rename = "prognosedDeparture",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prognosed_departure: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Departure delay in seconds.
    #[serde(
        rename = "departureDelay",
        default,
        with = "crate::util::lenient_i64",
        skip_serializing_if = "Option::is_none"
    )]
    pub departure_delay: Option<i64>,
    /// Realtime departure platform.
    #[serde(
        rename = "departurePlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub departure_platform: Option<String>,
    /// Scheduled departure platform.
    #[serde(
        rename = "plannedDeparturePlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub planned_departure_platform: Option<String>,
    /// Prognosed departure platform.
    #[serde(
        rename = "prognosedDeparturePlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prognosed_departure_platform: Option<String>,
    /// Departure prognosis class.
    #[serde(
        rename = "departurePrognosisType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub departure_prognosis_type: Option<PrognosisType>,
    /// Realtime arrival time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arrival: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Scheduled arrival time.
    #[serde(
        rename = "plannedArrival",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub planned_arrival: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Prognosed arrival time.
    #[serde(
        rename = "prognosedArrival",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prognosed_arrival: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Arrival delay in seconds.
    #[serde(
        rename = "arrivalDelay",
        default,
        with = "crate::util::lenient_i64",
        skip_serializing_if = "Option::is_none"
    )]
    pub arrival_delay: Option<i64>,
    /// Realtime arrival platform.
    #[serde(
        rename = "arrivalPlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub arrival_platform: Option<String>,
    /// Scheduled arrival platform.
    #[serde(
        rename = "plannedArrivalPlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub planned_arrival_platform: Option<String>,
    /// Prognosed arrival platform.
    #[serde(
        rename = "prognosedArrivalPlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prognosed_arrival_platform: Option<String>,
    /// Arrival prognosis class.
    #[serde(
        rename = "arrivalPrognosisType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub arrival_prognosis_type: Option<PrognosisType>,
    /// Intermediate stops of this leg (`stopovers=true`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stopovers: Vec<Stopover>,
    /// Remarks/hints/warnings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remarks: Vec<Remark>,
    /// Walking leg?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub walking: Option<bool>,
    /// Transfer leg (change within a station)?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transfer: Option<bool>,
    /// Distance in meters (walking/cycling/driving legs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance: Option<f64>,
    /// Publicly bookable?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
    /// Transfer into this leg guaranteed?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reachable: Option<bool>,
    /// Leg cancelled?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancelled: Option<bool>,
    /// Expected occupancy; open value set.
    #[serde(
        rename = "loadFactor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub load_factor: Option<String>,
    /// Cycle/headway info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cycle: Option<Cycle>,
    /// Alternative departures at [`Leg::origin`] (db profile).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternatives: Vec<Departure>,
    /// Geographic shape (`polylines=true`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub polyline: Option<Polyline>,
    /// Price of this leg, if priced individually.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<Price>,
    /// Schedule reference id (db-vendo).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schedule: Option<i64>,
    /// Current vehicle position (live legs).
    #[serde(
        rename = "currentLocation",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub current_location: Option<super::place::Location>,
    /// Check-in capable leg (db).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkin: Option<bool>,
    /// Days this leg is served (db-vendo `serviceDays`).
    #[serde(
        rename = "serviceDays",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub service_days: Option<Aliases::ServiceDays>,
    /// Days served (hafas-client style `scheduledDays`).
    #[serde(
        rename = "scheduledDays",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub scheduled_days: Option<Aliases::ServiceDays>,
}

/// A computed set of directions to get from A to B.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Journey {
    /// Journey ID, if the instance assigns one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Pass to [`crate::TransportRestClient::refresh_journey`] to obtain
    /// updated realtime data for this exact journey.
    #[serde(
        rename = "refreshToken",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub refresh_token: Option<String>,
    /// Legs of the journey in order.
    #[serde(default)]
    pub legs: Vec<Leg>,
    /// Remarks/hints/warnings affecting the whole journey.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remarks: Vec<Remark>,
    /// Total price, if requested & returned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<Price>,
    /// Cycle/headway info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cycle: Option<Cycle>,
    /// Days this journey is served (db-vendo `serviceDays`).
    #[serde(
        rename = "serviceDays",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub service_days: Option<Aliases::ServiceDays>,
    /// Days served (hafas-client style `scheduledDays`).
    #[serde(
        rename = "scheduledDays",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub scheduled_days: Option<Aliases::ServiceDays>,
}

/// A specific vehicle stopping at a set of stops at specific times
/// (the result of `GET /trips/{id}` and `GET /trips`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trip {
    /// Trip ID; use with [`crate::TransportRestClient::trip`].
    pub id: String,
    /// Where the trip starts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<Place>,
    /// Where the trip ends.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination: Option<Place>,
    /// Line the vehicle runs on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<Line>,
    /// Head-sign / terminus.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    /// Operating company.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<Operator>,
    /// Realtime departure time at [`Trip::origin`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub departure: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Scheduled departure time.
    #[serde(
        rename = "plannedDeparture",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub planned_departure: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Prognosed departure time.
    #[serde(
        rename = "prognosedDeparture",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prognosed_departure: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Departure delay in seconds.
    #[serde(
        rename = "departureDelay",
        default,
        with = "crate::util::lenient_i64",
        skip_serializing_if = "Option::is_none"
    )]
    pub departure_delay: Option<i64>,
    /// Realtime departure platform.
    #[serde(
        rename = "departurePlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub departure_platform: Option<String>,
    /// Scheduled departure platform.
    #[serde(
        rename = "plannedDeparturePlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub planned_departure_platform: Option<String>,
    /// Prognosed departure platform.
    #[serde(
        rename = "prognosedDeparturePlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prognosed_departure_platform: Option<String>,
    /// Departure prognosis class.
    #[serde(
        rename = "departurePrognosisType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub departure_prognosis_type: Option<PrognosisType>,
    /// Realtime arrival time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arrival: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Scheduled arrival time.
    #[serde(
        rename = "plannedArrival",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub planned_arrival: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Prognosed arrival time.
    #[serde(
        rename = "prognosedArrival",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prognosed_arrival: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Arrival delay in seconds.
    #[serde(
        rename = "arrivalDelay",
        default,
        with = "crate::util::lenient_i64",
        skip_serializing_if = "Option::is_none"
    )]
    pub arrival_delay: Option<i64>,
    /// Realtime arrival platform.
    #[serde(
        rename = "arrivalPlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub arrival_platform: Option<String>,
    /// Scheduled arrival platform.
    #[serde(
        rename = "plannedArrivalPlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub planned_arrival_platform: Option<String>,
    /// Prognosed arrival platform.
    #[serde(
        rename = "prognosedArrivalPlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prognosed_arrival_platform: Option<String>,
    /// Arrival prognosis class.
    #[serde(
        rename = "arrivalPrognosisType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub arrival_prognosis_type: Option<PrognosisType>,
    /// All stops of the trip (`stopovers=true`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stopovers: Vec<Stopover>,
    /// Remarks/hints/warnings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remarks: Vec<Remark>,
    /// Walking leg?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub walking: Option<bool>,
    /// Transfer leg?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transfer: Option<bool>,
    /// Distance in meters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance: Option<f64>,
    /// Publicly bookable?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
    /// Trip cancelled?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancelled: Option<bool>,
    /// Expected occupancy; open value set.
    #[serde(
        rename = "loadFactor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub load_factor: Option<String>,
    /// Cycle/headway info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cycle: Option<Cycle>,
    /// Alternative departures (db profile).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternatives: Vec<Departure>,
    /// Geographic shape (`polyline=true`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub polyline: Option<Polyline>,
    /// Price of the trip.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<Price>,
    /// Schedule reference id (db-vendo).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schedule: Option<i64>,
    /// Current vehicle position.
    #[serde(
        rename = "currentLocation",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub current_location: Option<super::place::Location>,
    /// Check-in capable trip (db).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkin: Option<bool>,
    /// Days this trip is served (db-vendo `serviceDays`).
    #[serde(
        rename = "serviceDays",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub service_days: Option<Aliases::ServiceDays>,
    /// Days served (hafas-client style `scheduledDays`).
    #[serde(
        rename = "scheduledDays",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub scheduled_days: Option<Aliases::ServiceDays>,
}

/// A departure board or arrival board entry
/// (`Alternative` in the upstream OpenAPI spec).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Departure {
    /// ID of the arriving/departing trip.
    #[serde(rename = "tripId", default, skip_serializing_if = "Option::is_none")]
    pub trip_id: Option<String>,
    /// The stop this entry belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop: Option<StopOrStation>,
    /// Current vehicle position (radar movements).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<super::place::Location>,
    /// Line of the vehicle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<Line>,
    /// Head-sign / terminus (departures).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    /// Where the vehicle came from (arrivals).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
    /// Full origin place, if returned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<Place>,
    /// Full destination place, if returned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination: Option<Place>,
    /// Realtime time of the event; `None` when cancelled.
    #[serde(rename = "when", default, skip_serializing_if = "Option::is_none")]
    pub when: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Scheduled time of the event.
    #[serde(
        rename = "plannedWhen",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub planned_when: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Prognosed time of the event.
    #[serde(
        rename = "prognosedWhen",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prognosed_when: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Delay in seconds; `None` when unknown.
    #[serde(
        default,
        with = "crate::util::lenient_i64",
        skip_serializing_if = "Option::is_none"
    )]
    pub delay: Option<i64>,
    /// Realtime platform; may change or be `None` when cancelled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    /// Scheduled platform.
    #[serde(
        rename = "plannedPlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub planned_platform: Option<String>,
    /// Prognosed platform.
    #[serde(
        rename = "prognosedPlatform",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prognosed_platform: Option<String>,
    /// Prognosis class.
    #[serde(
        rename = "prognosisType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prognosis_type: Option<PrognosisType>,
    /// Remarks/hints/warnings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remarks: Vec<Remark>,
    /// Event cancelled?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancelled: Option<bool>,
    /// Expected occupancy; open value set.
    #[serde(
        rename = "loadFactor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub load_factor: Option<String>,
    /// Stops already served (`stopovers=true`, departures).
    #[serde(
        rename = "previousStopovers",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub previous_stopovers: Vec<Stopover>,
    /// Upcoming stops (`stopovers=true`).
    #[serde(
        rename = "nextStopovers",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub next_stopovers: Vec<Stopover>,
    /// Radar frames.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frames: Vec<Frame>,
    /// Geographic shape.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub polyline: Option<Polyline>,
    /// Current position of the trip on its route.
    #[serde(
        rename = "currentTripPosition",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub current_trip_position: Option<super::place::Location>,
}

/// Radar frame: interpolated vehicle position between two places at offset `t`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    /// Previous place.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<Place>,
    /// Next place.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination: Option<Place>,
    /// Interpolation offset within the frame.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub t: Option<f64>,
}

/// A vehicle currently moving inside a radar bounding box.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Movement {
    /// Head-sign / terminus.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    /// Trip ID.
    #[serde(rename = "tripId", default, skip_serializing_if = "Option::is_none")]
    pub trip_id: Option<String>,
    /// Line the vehicle runs on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<Line>,
    /// Current position.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<super::place::Location>,
    /// Upcoming stops.
    #[serde(
        rename = "nextStopovers",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub next_stopovers: Vec<Stopover>,
    /// Predicted frames.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frames: Vec<Frame>,
    /// Geographic shape.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub polyline: Option<Polyline>,
}

/// Reachability result: travel duration and the stations reachable in it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReachableDuration {
    /// Travel duration in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,
    /// Stations reachable within [`ReachableDuration::duration`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stations: Vec<super::place::LocationResult>,
}
