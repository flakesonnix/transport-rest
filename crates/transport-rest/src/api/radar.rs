//! `GET /radar` – vehicles inside a bounding box (capability-gated).

use crate::datetime::{DateTime, FixedOffset};
use crate::error::{InvalidParameterError, TransportRestError};
use crate::models::RadarResponse;
use crate::products::ProductSelection;
use crate::request::Query;
use crate::{ClientState, TransportRestClient};

/// Builder for radar queries.
#[must_use = "builders do nothing until you call .get()"]
#[derive(Debug, Clone)]
pub struct RadarBuilder {
    state: std::sync::Arc<ClientState>,
    north: Option<f64>,
    west: Option<f64>,
    south: Option<f64>,
    east: Option<f64>,
    results: Option<i64>,
    frames: Option<i64>,
    duration: Option<i64>,
    polylines: Option<bool>,
    when: Option<DateTime<FixedOffset>>,
    language: Option<String>,
    products: Option<ProductSelection>,
}

impl RadarBuilder {
    pub(crate) fn new(state: std::sync::Arc<ClientState>) -> Self {
        Self {
            state,
            north: None,
            west: None,
            south: None,
            east: None,
            results: None,
            frames: None,
            duration: None,
            polylines: None,
            when: None,
            language: None,
            products: None,
        }
    }

    /// Required: northern edge latitude.
    pub fn north(mut self, north: f64) -> Self {
        self.north = Some(north);
        self
    }

    /// Required: western edge longitude.
    pub fn west(mut self, west: f64) -> Self {
        self.west = Some(west);
        self
    }

    /// Required: southern edge latitude.
    pub fn south(mut self, south: f64) -> Self {
        self.south = Some(south);
        self
    }

    /// Required: eastern edge longitude.
    pub fn east(mut self, east: f64) -> Self {
        self.east = Some(east);
        self
    }

    /// Maximum number of vehicles (default 256).
    pub fn results(mut self, results: i64) -> Self {
        self.results = Some(results);
        self
    }

    /// Number of predicted frames (default 3).
    pub fn frames(mut self, frames: i64) -> Self {
        self.frames = Some(frames);
        self
    }

    /// Compute frames for the next n seconds (default 20).
    pub fn duration(mut self, seconds: i64) -> Self {
        self.duration = Some(seconds);
        self
    }

    /// Include shapes per frame.
    pub fn polylines(mut self, polylines: bool) -> Self {
        self.polylines = Some(polylines);
        self
    }

    /// Compute positions relative to this time.
    pub fn when(mut self, when: DateTime<FixedOffset>) -> Self {
        self.when = Some(when);
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
    pub async fn get(self) -> Result<RadarResponse, TransportRestError> {
        let client = TransportRestClient { state: self.state };
        client.check_capability(crate::Capability::Radar)?;

        let (north, west, south, east) = match (self.north, self.west, self.south, self.east) {
            (Some(n), Some(w), Some(s), Some(e)) => (n, w, s, e),
            _ => {
                return Err(TransportRestError::InvalidParameter(
                    InvalidParameterError::other("north, west, south and east are all required"),
                ))
            }
        };
        if !(south <= north && west <= east) {
            return Err(TransportRestError::InvalidParameter(
                InvalidParameterError::other(
                    "bounding box is invalid: require south <= north and west <= east",
                ),
            ));
        }
        let mut q = Query::new();
        q.push("north", north.to_string());
        q.push("west", west.to_string());
        q.push("south", south.to_string());
        q.push("east", east.to_string());
        q.opt("results", self.results);
        q.opt("frames", self.frames);
        q.opt("duration", self.duration);
        q.opt("polylines", self.polylines);
        if let Some(t) = self.when {
            q.push("when", t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
        }
        q.opt("language", self.language);
        if let Some(products) = self.products.filter(|p| !p.is_empty()) {
            products.encode(&mut q);
        }
        client.get_json("/radar", q).await
    }
}
