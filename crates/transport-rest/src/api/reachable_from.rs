//! `GET /stops/reachable-from` – isochrone-style reachability (capability-gated).

use crate::datetime::{DateTime, FixedOffset};
use crate::error::TransportRestError;
use crate::models::ReachableFromResponse;
use crate::products::ProductSelection;
use crate::request::Query;
use crate::{ClientState, TransportRestClient};

/// Builder for reachable-from queries.
#[must_use = "builders do nothing until you call .get()"]
#[derive(Debug, Clone)]
pub struct ReachableFromBuilder {
    state: std::sync::Arc<ClientState>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    when: Option<DateTime<FixedOffset>>,
    max_transfers: Option<i64>,
    max_duration: Option<i64>,
    polylines: Option<bool>,
    language: Option<String>,
    products: Option<ProductSelection>,
}

impl ReachableFromBuilder {
    pub(crate) fn new(state: std::sync::Arc<ClientState>) -> Self {
        Self {
            state,
            latitude: None,
            longitude: None,
            when: None,
            max_transfers: None,
            max_duration: None,
            polylines: None,
            language: None,
            products: None,
        }
    }

    /// Required: origin latitude.
    pub fn latitude(mut self, latitude: f64) -> Self {
        self.latitude = Some(latitude);
        self
    }

    /// Required: origin longitude.
    pub fn longitude(mut self, longitude: f64) -> Self {
        self.longitude = Some(longitude);
        self
    }

    /// Compute reachability from this time (default now).
    pub fn when(mut self, when: DateTime<FixedOffset>) -> Self {
        self.when = Some(when);
        self
    }

    /// Maximum number of transfers (default 5).
    pub fn max_transfers(mut self, transfers: i64) -> Self {
        self.max_transfers = Some(transfers);
        self
    }

    /// Maximum travel duration in minutes (default 20).
    pub fn max_duration(mut self, minutes: i64) -> Self {
        self.max_duration = Some(minutes);
        self
    }

    /// Include shapes per leg.
    pub fn polylines(mut self, polylines: bool) -> Self {
        self.polylines = Some(polylines);
        self
    }

    /// Language of result texts.
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Filter included transport products.
    pub fn products<F: FnOnce(ProductSelection) -> ProductSelection>(mut self, f: F) -> Self {
        self.products = Some(f(ProductSelection::default()));
        self
    }

    /// Execute the request.
    pub async fn get(self) -> Result<ReachableFromResponse, TransportRestError> {
        let client = TransportRestClient { state: self.state };
        client.check_capability(crate::Capability::ReachableFrom)?;

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
        if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
            return Err(TransportRestError::InvalidParameter(
                crate::error::InvalidParameterError::other(
                    "coordinates out of range",
                ),
            ));
        }
        let mut q = Query::new();
        q.push("latitude", lat.to_string());
        q.push("longitude", lon.to_string());
        if let Some(t) = self.when {
            q.push("when", t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
        }
        q.opt("maxTransfers", self.max_transfers);
        q.opt("maxDuration", self.max_duration);
        q.opt("polylines", self.polylines);
        q.opt("language", self.language);
        if let Some(products) = self.products.filter(|p| !p.is_empty()) {
            products.encode(&mut q);
        }
        client.get_json("/stops/reachable-from", q).await
    }
}
