//! `GET /locations` – search stops/stations, POIs and addresses.

use crate::error::TransportRestError;
use crate::models::{LocationsResponse, LocationResult};
use crate::request::Query;
use crate::{ClientState, TransportRestClient};

/// Builder for the locations search endpoint.
#[must_use = "builders do nothing until you call .get()"]
#[derive(Debug, Clone)]
pub struct LocationsBuilder {
    pub(crate) state: std::sync::Arc<ClientState>,
    query: Option<String>,
    fuzzy: Option<bool>,
    results: Option<i64>,
    stops: Option<bool>,
    addresses: Option<bool>,
    poi: Option<bool>,
    lines_of_stops: Option<bool>,
    language: Option<String>,
}

impl LocationsBuilder {
    pub(crate) fn new(state: std::sync::Arc<ClientState>) -> Self {
        Self {
            state,
            query: None,
            fuzzy: None,
            results: None,
            stops: None,
            addresses: None,
            poi: None,
            lines_of_stops: None,
            language: None,
        }
    }

    /// Set the required search term.
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// Find more than exact matches? (default: server-defined)
    pub fn fuzzy(mut self, fuzzy: bool) -> Self {
        self.fuzzy = Some(fuzzy);
        self
    }

    /// Maximum number of results.
    pub fn results(mut self, results: i64) -> Self {
        self.results = Some(results);
        self
    }

    /// Include stops/stations?
    pub fn stops(mut self, stops: bool) -> Self {
        self.stops = Some(stops);
        self
    }

    /// Include addresses?
    pub fn addresses(mut self, addresses: bool) -> Self {
        self.addresses = Some(addresses);
        self
    }

    /// Include points of interest?
    pub fn poi(mut self, poi: bool) -> Self {
        self.poi = Some(poi);
        self
    }

    /// Include the lines serving each returned stop.
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
        let query = match self.query.as_deref() {
            Some(q) if !q.trim().is_empty() => q.to_owned(),
            _ => {
                return Err(TransportRestError::InvalidParameter(
                    crate::error::InvalidParameterError::new("query", "a non-empty search term is required"),
                ))
            }
        };
        let mut q = Query::new();
        q.push("query", query);
        q.opt("fuzzy", self.fuzzy);
        q.opt("results", self.results);
        q.opt("stops", self.stops);
        q.opt("addresses", self.addresses);
        q.opt("poi", self.poi);
        q.opt("linesOfStops", self.lines_of_stops);
        q.opt("language", self.language);

        TransportRestClient { state: self.state }
            .get_json::<Vec<LocationResult>>("/locations", q)
            .await
    }
}
