//! `GET /trips/{id}` & `GET /trips` – trip lookups.

use crate::datetime::{DateTime, FixedOffset};
use crate::error::{InvalidParameterError, TransportRestError};
use crate::models::{TripResponse, TripsResponse};
use crate::products::ProductSelection;
use crate::request::Query;
use crate::util::encode_path_segment;
use crate::{ClientState, TransportRestClient};

/// Builder for `GET /trips/{id}`.
#[must_use = "builders do nothing until you call .get()"]
#[derive(Debug, Clone)]
pub struct TripBuilder {
    state: std::sync::Arc<ClientState>,
    id: String,
    stopovers: Option<bool>,
    remarks: Option<bool>,
    polyline: Option<bool>,
    language: Option<String>,
}

impl TripBuilder {
    pub(crate) fn new(state: std::sync::Arc<ClientState>, id: String) -> Self {
        Self {
            state,
            id,
            stopovers: None,
            remarks: None,
            polyline: None,
            language: None,
        }
    }

    /// Include all stopovers (default on).
    pub fn stopovers(mut self, stopovers: bool) -> Self {
        self.stopovers = Some(stopovers);
        self
    }

    /// Include hints & warnings (default on).
    pub fn remarks(mut self, remarks: bool) -> Self {
        self.remarks = Some(remarks);
        self
    }

    /// Include the geographic shape of the trip.
    pub fn polyline(mut self, polyline: bool) -> Self {
        self.polyline = Some(polyline);
        self
    }

    /// Language of result texts.
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Execute the request.
    pub async fn get(self) -> Result<TripResponse, TransportRestError> {
        if self.id.trim().is_empty() {
            return Err(TransportRestError::InvalidParameter(
                InvalidParameterError::new("id", "trip ID must not be empty"),
            ));
        }
        let mut q = Query::new();
        q.opt("stopovers", self.stopovers);
        q.opt("remarks", self.remarks);
        q.opt("polyline", self.polyline);
        q.opt("language", self.language);

        let path = format!("/trips/{}", encode_path_segment(&self.id));
        TransportRestClient { state: self.state }
            .get_json(&path, q)
            .await
    }
}

/// Builder for `GET /trips` – find trips by name (capability-gated).
#[must_use = "builders do nothing until you call .get()"]
#[derive(Debug, Clone)]
pub struct TripsByNameBuilder {
    state: std::sync::Arc<ClientState>,
    query: Option<String>,
    when: Option<DateTime<FixedOffset>>,
    from_when: Option<DateTime<FixedOffset>>,
    until_when: Option<DateTime<FixedOffset>>,
    only_currently_running: Option<bool>,
    currently_stopping_at: Option<String>,
    line_name: Option<String>,
    operator_names: Option<Vec<String>>,
    products: Option<ProductSelection>,
}

impl TripsByNameBuilder {
    pub(crate) fn new(state: std::sync::Arc<ClientState>, query: String) -> Self {
        Self {
            state,
            query: Some(query),
            when: None,
            from_when: None,
            until_when: None,
            only_currently_running: None,
            currently_stopping_at: None,
            line_name: None,
            operator_names: None,
            products: None,
        }
    }

    /// Trip name to search for (`*` matches everything).
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// Only trips around this time.
    pub fn when(mut self, when: DateTime<FixedOffset>) -> Self {
        self.when = Some(when);
        self
    }

    /// Earliest departure of matched trips.
    pub fn from_when(mut self, from: DateTime<FixedOffset>) -> Self {
        self.from_when = Some(from);
        self
    }

    /// Latest departure of matched trips.
    pub fn until_when(mut self, until: DateTime<FixedOffset>) -> Self {
        self.until_when = Some(until);
        self
    }

    /// Restrict to trips currently running (default on).
    pub fn only_currently_running(mut self, yes: bool) -> Self {
        self.only_currently_running = Some(yes);
        self
    }

    /// Only trips currently stopping at this stop ID.
    pub fn currently_stopping_at(mut self, stop_id: impl Into<String>) -> Self {
        self.currently_stopping_at = Some(stop_id.into());
        self
    }

    /// Filter by line name.
    pub fn line_name(mut self, line_name: impl Into<String>) -> Self {
        self.line_name = Some(line_name.into());
        self
    }

    /// Filter by operator names.
    pub fn operator_names<I: IntoIterator<Item = impl Into<String>>>(mut self, names: I) -> Self {
        self.operator_names = Some(names.into_iter().map(Into::into).collect());
        self
    }

    /// Filter included transport products.
    pub fn products<F: FnOnce(ProductSelection) -> ProductSelection>(mut self, f: F) -> Self {
        self.products = Some(f(ProductSelection::default()));
        self
    }

    /// Execute the request.
    pub async fn get(self) -> Result<TripsResponse, TransportRestError> {
        let client = TransportRestClient { state: self.state };
        client.check_capability(crate::Capability::TripsByName)?;
        let mut q = Query::new();
        q.opt("query", self.query);
        if let Some(t) = self.when {
            q.push("when", t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
        }
        if let Some(t) = self.from_when {
            q.push(
                "fromWhen",
                t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            );
        }
        if let Some(t) = self.until_when {
            q.push(
                "untilWhen",
                t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            );
        }
        q.opt("onlyCurrentlyRunning", self.only_currently_running);
        q.opt("currentlyStoppingAt", self.currently_stopping_at);
        q.opt("lineName", self.line_name);
        if let Some(names) = self.operator_names.filter(|n| !n.is_empty()) {
            // hafas-rest-api expects cli-native array encoding.
            q.push("operatorNames", format!("[{}]", names.join(",")));
        }
        if let Some(products) = self.products.filter(|p| !p.is_empty()) {
            products.encode(&mut q);
        }
        client.get_json("/trips", q).await
    }
}
