//! `GET /locations/nearby` – stops/stations & POIs close to coordinates.

use crate::error::TransportRestError;
use crate::models::{LocationsResponse, LocationResult};
use crate::request::Query;
use crate::{ClientState, TransportRestClient};

/// Builder for the nearby endpoint.
#[must_use = "builders do nothing until you call .get()"]
#[derive(Debug, Clone)]
pub struct NearbyBuilder {
    state: std::sync::Arc<ClientState>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    results: Option<i64>,
    distance: Option<i64>,
    stops: Option<bool>,
    poi: Option<bool>,
    lines_of_stops: Option<bool>,
    language: Option<String>,
}

impl NearbyBuilder {
    pub(crate) fn new(state: std::sync::Arc<ClientState>) -> Self {
        Self {
            state,
            latitude: None,
            longitude: None,
            results: None,
            distance: None,
            stops: None,
            poi: None,
            lines_of_stops: None,
            language: None,
        }
    }

    /// Required: geographic latitude in degrees.
    pub fn latitude(mut self, latitude: f64) -> Self {
        self.latitude = Some(latitude);
        self
    }

    /// Required: geographic longitude in degrees.
    pub fn longitude(mut self, longitude: f64) -> Self {
        self.longitude = Some(longitude);
        self
    }

    /// Maximum number of results.
    pub fn results(mut self, results: i64) -> Self {
        self.results = Some(results);
        self
    }

    /// Maximum walking distance in meters.
    pub fn distance(mut self, distance: i64) -> Self {
        self.distance = Some(distance);
        self
    }

    /// Include stops/stations?
    pub fn stops(mut self, stops: bool) -> Self {
        self.stops = Some(stops);
        self
    }

    /// Include points of interest?
    pub fn poi(mut self, poi: bool) -> Self {
        self.poi = Some(poi);
        self
    }

    /// Include lines serving each returned stop.
    pub fn lines_of_stops(mut self, lines_of_stops: bool) -> Self {
        self.lines_of_stops = Some(lines_of_stops);
        self
    }

    /// Language of result names.
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Execute the request.
    pub async fn get(self) -> Result<LocationsResponse, TransportRestError> {
        let (lat, lon) = match (self.latitude, self.longitude) {
            (Some(lat), Some(lon)) => (lat, lon),
            _ => {
                return Err(TransportRestError::InvalidParameter(
                    crate::error::InvalidParameterError::other(
                        "latitude and longitude are both required",
                    ),
                ))
            }
        };
        if !(-90.0..=90.0).contains(&lat) {
            return Err(TransportRestError::InvalidParameter(
                crate::error::InvalidParameterError::new("latitude", "must be within [-90, 90]"),
            ));
        }
        if !(-180.0..=180.0).contains(&lon) {
            return Err(TransportRestError::InvalidParameter(
                crate::error::InvalidParameterError::new("longitude", "must be within [-180, 180]"),
            ));
        }
        let mut q = Query::new();
        q.push("latitude", lat.to_string());
        q.push("longitude", lon.to_string());
        q.opt("results", self.results);
        q.opt("distance", self.distance);
        q.opt("stops", self.stops);
        q.opt("poi", self.poi);
        q.opt("linesOfStops", self.lines_of_stops);
        q.opt("language", self.language);

        TransportRestClient { state: self.state }
            .get_json::<Vec<LocationResult>>("/locations/nearby", q)
            .await
    }
}
