//! Static station directory endpoints (`/stations`, `/stations/{id}`, `/stops`).

use crate::error::TransportRestError;
use crate::models::{LocationResult, LocationsResponse, Station};
use crate::request::Query;
use crate::util::encode_path_segment;
use crate::{ClientState, TransportRestClient};

/// Builder for `GET /stations` – static station search (DB instance).
#[must_use = "builders do nothing until you call .get()"]
#[derive(Debug, Clone)]
pub struct StationsBuilder {
    state: std::sync::Arc<ClientState>,
    query: Option<String>,
    results: Option<i64>,
    fuzzy: Option<bool>,
    completion: Option<bool>,
}

impl StationsBuilder {
    pub(crate) fn new(state: std::sync::Arc<ClientState>) -> Self {
        Self {
            state,
            query: None,
            results: None,
            fuzzy: None,
            completion: None,
        }
    }

    /// Search term; without it all stations are returned.
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// Maximum number of results (only with `query`; default 3).
    pub fn results(mut self, results: i64) -> Self {
        self.results = Some(results);
        self
    }

    /// Allow fuzzy matching?
    pub fn fuzzy(mut self, fuzzy: bool) -> Self {
        self.fuzzy = Some(fuzzy);
        self
    }

    /// Match by prefix instead of full name?
    pub fn completion(mut self, completion: bool) -> Self {
        self.completion = Some(completion);
        self
    }

    /// Execute the request.
    pub async fn get(self) -> Result<Vec<Station>, TransportRestError> {
        let client = TransportRestClient { state: self.state };
        client.check_capability(crate::Capability::Stations)?;
        let mut q = Query::new();
        q.opt("query", self.query);
        q.opt("results", self.results);
        q.opt("fuzzy", self.fuzzy);
        q.opt("completion", self.completion);
        client.get_json::<Vec<Station>>("/stations", q).await
    }
}

/// Builder for `GET /stations/{id}` (DB instance).
#[must_use = "builders do nothing until you call .get()"]
#[derive(Debug, Clone)]
pub struct StationBuilder {
    state: std::sync::Arc<ClientState>,
    id: String,
}

impl StationBuilder {
    pub(crate) fn new(state: std::sync::Arc<ClientState>, id: String) -> Self {
        Self { state, id }
    }

    /// Execute the request.
    pub async fn get(self) -> Result<Station, TransportRestError> {
        let client = TransportRestClient { state: self.state };
        client.check_capability(crate::Capability::Stations)?;
        if self.id.trim().is_empty() {
            return Err(TransportRestError::InvalidParameter(
                crate::error::InvalidParameterError::new("id", "station ID must not be empty"),
            ));
        }
        let path = format!("/stations/{}", encode_path_segment(&self.id));
        client.get_json::<Station>(&path, Query::new()).await
    }
}

/// Builder for `GET /stops` – static stop name search (BVG/VBB instances).
#[must_use = "builders do nothing until you call .get()"]
#[derive(Debug, Clone)]
pub struct StopsSearchBuilder {
    state: std::sync::Arc<ClientState>,
    query: Option<String>,
    limit: Option<i64>,
    fuzzy: Option<bool>,
    completion: Option<bool>,
}

impl StopsSearchBuilder {
    pub(crate) fn new(state: std::sync::Arc<ClientState>) -> Self {
        Self {
            state,
            query: None,
            limit: None,
            fuzzy: None,
            completion: None,
        }
    }

    /// Name filter (e.g. `"mehringd"` with completion, or the full name
    /// without).
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// Maximum number of results (default 5).
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Find other than exact matches?
    pub fn fuzzy(mut self, fuzzy: bool) -> Self {
        self.fuzzy = Some(fuzzy);
        self
    }

    /// Search by prefix? (default on)
    pub fn completion(mut self, completion: bool) -> Self {
        self.completion = Some(completion);
        self
    }

    /// Execute the request.
    pub async fn get(self) -> Result<LocationsResponse, TransportRestError> {
        let client = TransportRestClient { state: self.state };
        client.check_capability(crate::Capability::StopsSearch)?;
        let mut q = Query::new();
        q.opt("query", self.query);
        q.opt("limit", self.limit);
        q.opt("fuzzy", self.fuzzy);
        q.opt("completion", self.completion);
        client.get_json::<Vec<LocationResult>>("/stops", q).await
    }
}
