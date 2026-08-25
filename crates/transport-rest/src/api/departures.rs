//! `GET /stops/{id}/departures` & `GET /stops/{id}/arrivals` – boards.

use crate::error::{InvalidParameterError, TransportRestError};
use crate::models::{ArrivalsResponse, DeparturesResponse};
use crate::products::ProductSelection;
use crate::request::Query;
use crate::util::encode_path_segment;
use crate::{ClientState, TransportRestClient};

fn validate_id(id: &str) -> Result<(), TransportRestError> {
    if id.trim().is_empty() {
        Err(TransportRestError::InvalidParameter(
            InvalidParameterError::new("stop_id", "must not be empty"),
        ))
    } else {
        Ok(())
    }
}

/// Builder for departure board queries.
#[must_use = "builders do nothing until you call .get()"]
#[derive(Debug, Clone)]
pub struct DeparturesBuilder {
    pub(crate) base: BoardBase,
}

/// Builder for arrival board queries.
#[must_use = "builders do nothing until you call .get()"]
#[derive(Debug, Clone)]
pub struct ArrivalsBuilder {
    pub(crate) base: BoardBase,
}

#[derive(Debug, Clone)]
pub(crate) struct BoardBase {
    pub(crate) state: std::sync::Arc<ClientState>,
    pub(crate) stop_id: String,
    pub(crate) when: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    pub(crate) direction: Option<String>,
    pub(crate) duration: Option<i64>,
    pub(crate) results: Option<i64>,
    pub(crate) stopovers: Option<bool>,
    pub(crate) include_related_stations: Option<bool>,
    pub(crate) lines_of_stops: Option<bool>,
    pub(crate) remarks: Option<bool>,
    pub(crate) language: Option<String>,
    pub(crate) more_stops: Option<Vec<String>>,
    pub(crate) products: Option<ProductSelection>,
}

macro_rules! board_setters {
    ($name:ident) => {
        impl $name {
            /// Query the board at this date/time instead of now.
            pub fn when(
                mut self,
                when: crate::datetime::DateTime<crate::datetime::FixedOffset>,
            ) -> Self {
                self.base.when = Some(when);
                self
            }

            /// Only show vehicles heading towards this stop (ID or name).
            ///
            /// Note: only honored by some provider profiles.
            pub fn direction(mut self, direction: impl Into<String>) -> Self {
                self.base.direction = Some(direction.into());
                self
            }

            /// Show departures/arrivals for the next n minutes.
            pub fn duration(mut self, minutes: i64) -> Self {
                self.base.duration = Some(minutes);
                self
            }

            /// Maximum number of entries.
            pub fn results(mut self, results: i64) -> Self {
                self.base.results = Some(results);
                self
            }

            /// Include previous/next stopovers of each vehicle.
            pub fn stopovers(mut self, stopovers: bool) -> Self {
                self.base.stopovers = Some(stopovers);
                self
            }

            /// Include departures/arrivals of related stations (default on).
            pub fn include_related_stations(mut self, yes: bool) -> Self {
                self.base.include_related_stations = Some(yes);
                self
            }

            /// Include lines serving each entry's stop.
            pub fn lines_of_stops(mut self, lines_of_stops: bool) -> Self {
                self.base.lines_of_stops = Some(lines_of_stops);
                self
            }

            /// Include hints & warnings (default on).
            pub fn remarks(mut self, remarks: bool) -> Self {
                self.base.remarks = Some(remarks);
                self
            }

            /// Language of result texts.
            pub fn language(mut self, language: impl Into<String>) -> Self {
                self.base.language = Some(language.into());
                self
            }

            /// Also fetch boards for these additional stop IDs
            /// (DB instance; up to nine comma-separated EVA numbers,
            /// unsupported with the `dbnav`/`dbweb` profiles).
            pub fn more_stops<I: IntoIterator<Item = impl Into<String>>>(
                mut self,
                stops: I,
            ) -> Self {
                self.base.more_stops =
                    Some(stops.into_iter().map(Into::into).collect());
                self
            }

            /// Filter included transport products.
            pub fn products<F: FnOnce(crate::products::ProductSelection) -> crate::products::ProductSelection>(
                mut self,
                f: F,
            ) -> Self {
                self.base.products = Some(f(crate::products::ProductSelection::default()));
                self
            }
        }
    };
}

board_setters!(DeparturesBuilder);
board_setters!(ArrivalsBuilder);

impl DeparturesBuilder {
    pub(crate) fn departures(state: std::sync::Arc<ClientState>, stop_id: String) -> Self {
        Self {
            base: BoardBase::new(state, stop_id),
        }
    }

    /// Execute the request.
    pub async fn get(self) -> Result<DeparturesResponse, TransportRestError> {
        validate_id(&self.base.stop_id)?;
        let path = format!("/stops/{}/departures", encode_path_segment(&self.base.stop_id));
        let (state, query) = self.base.into_parts();
        TransportRestClient { state }
            .get_json(&path, query)
            .await
    }
}

impl ArrivalsBuilder {
    pub(crate) fn arrivals(state: std::sync::Arc<ClientState>, stop_id: String) -> Self {
        Self {
            base: BoardBase::new(state, stop_id),
        }
    }

    /// Execute the request.
    pub async fn get(self) -> Result<ArrivalsResponse, TransportRestError> {
        validate_id(&self.base.stop_id)?;
        let path = format!("/stops/{}/arrivals", encode_path_segment(&self.base.stop_id));
        let (state, query) = self.base.into_parts();
        TransportRestClient { state }
            .get_json(&path, query)
            .await
    }
}

impl BoardBase {
    pub(crate) fn new(state: std::sync::Arc<ClientState>, stop_id: String) -> Self {
        Self {
            state,
            stop_id,
            when: None,
            direction: None,
            duration: None,
            results: None,
            stopovers: None,
            include_related_stations: None,
            lines_of_stops: None,
            remarks: None,
            language: None,
            more_stops: None,
            products: None,
        }
    }

    pub(crate) fn into_parts(self) -> (std::sync::Arc<ClientState>, Query) {
        (self.state.clone(), self.into_query())
    }

    pub(crate) fn into_query(self) -> Query {
        let mut q = Query::new();
        if let Some(when) = self.when {
            q.push("when", when.to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
        }
        q.opt("direction", self.direction);
        q.opt("duration", self.duration);
        q.opt("results", self.results);
        q.opt("stopovers", self.stopovers);
        q.opt("includeRelatedStations", self.include_related_stations);
        q.opt("linesOfStops", self.lines_of_stops);
        q.opt("remarks", self.remarks);
        q.opt("language", self.language);
        if let Some(stops) = self.more_stops.filter(|s| !s.is_empty()) {
            q.push("moreStops", stops.join(","));
        }
        if let Some(products) = self.products.filter(|p| !p.is_empty()) {
            products.encode(&mut q);
        }
        q
    }
}
