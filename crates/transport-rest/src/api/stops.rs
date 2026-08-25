//! `GET /stops/{id}` – fetch one stop/station.

use crate::error::TransportRestError;
use crate::models::StopOrStation;
use crate::request::Query;
use crate::util::encode_path_segment;
use crate::{ClientState, TransportRestClient};

/// Builder for the stop-by-ID endpoint.
#[must_use = "builders do nothing until you call .get()"]
#[derive(Debug, Clone)]
pub struct StopBuilder {
    state: std::sync::Arc<ClientState>,
    id: String,
    lines_of_stops: Option<bool>,
    language: Option<String>,
}

impl StopBuilder {
    pub(crate) fn new(state: std::sync::Arc<ClientState>, id: String) -> Self {
        Self {
            state,
            id,
            lines_of_stops: None,
            language: None,
        }
    }

    /// Include the lines serving this stop.
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
    pub async fn get(self) -> Result<StopOrStation, TransportRestError> {
        if self.id.trim().is_empty() {
            return Err(TransportRestError::InvalidParameter(
                crate::error::InvalidParameterError::new("id", "stop ID must not be empty"),
            ));
        }
        let mut q = Query::new();
        q.opt("linesOfStops", self.lines_of_stops);
        q.opt("language", self.language);

        TransportRestClient { state: self.state }
            .get_json(&format!("/stops/{}", encode_path_segment(&self.id)), q)
            .await
    }
}
