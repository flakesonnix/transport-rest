//! `GET /journeys` & `GET /journeys/{ref}` – route search and refresh.

use crate::datetime::{DateTime, FixedOffset};
use crate::error::{InvalidParameterError, TransportRestError};
use crate::models::{JourneyResponse, JourneysResponse};
use crate::products::ProductSelection;
use crate::request::{JourneyPlace, Query};
use crate::util::encode_path_segment;
use crate::{ClientState, TransportRestClient};

/// Builder for the journey search endpoint.
///
/// `from` and `to` are mandatory; either `departure` or `arrival` may be set,
/// as may `earlier_than`/`later_than` for pagination (but not combined with
/// departure/arrival).
#[must_use = "builders do nothing until you call .get()"]
#[derive(Debug, Clone)]
pub struct JourneysBuilder {
    state: std::sync::Arc<ClientState>,
    from: JourneyPlace,
    to: JourneyPlace,
    via: Option<JourneyPlace>,
    departure: Option<DateTime<FixedOffset>>,
    arrival: Option<DateTime<FixedOffset>>,
    earlier_than: Option<String>,
    later_than: Option<String>,
    results: Option<i64>,
    stopovers: Option<bool>,
    transfers: Option<i64>,
    transfer_time: Option<i64>,
    accessibility: Option<String>,
    bike: Option<bool>,
    start_with_walking: Option<bool>,
    walking_speed: Option<String>,
    tickets: Option<bool>,
    polylines: Option<bool>,
    sub_stops: Option<bool>,
    entrances: Option<bool>,
    remarks: Option<bool>,
    scheduled_days: Option<bool>,
    not_only_fast_routes: Option<bool>,
    bestprice: Option<bool>,
    deutschland_ticket_connections_only: Option<bool>,
    loyalty_card: Option<String>,
    first_class: Option<bool>,
    age: Option<i64>,
    age_group: Option<String>,
    routing_mode: Option<String>,
    products: Option<ProductSelection>,
}

impl JourneysBuilder {
    pub(crate) fn new(
        state: std::sync::Arc<ClientState>,
        from: JourneyPlace,
        to: JourneyPlace,
    ) -> Self {
        Self {
            state,
            from,
            to,
            via: None,
            departure: None,
            arrival: None,
            earlier_than: None,
            later_than: None,
            results: None,
            stopovers: None,
            transfers: None,
            transfer_time: None,
            accessibility: None,
            bike: None,
            start_with_walking: None,
            walking_speed: None,
            tickets: None,
            polylines: None,
            sub_stops: None,
            entrances: None,
            remarks: None,
            scheduled_days: None,
            not_only_fast_routes: None,
            bestprice: None,
            deutschland_ticket_connections_only: None,
            loyalty_card: None,
            first_class: None,
            age: None,
            age_group: None,
            routing_mode: None,
            products: None,
        }
    }

    /// Route through this place.
    pub fn via(mut self, via: impl Into<JourneyPlace>) -> Self {
        self.via = Some(via.into());
        self
    }

    /// Search journeys departing at this time (mutually exclusive with
    /// `arrival` and pagination refs).
    pub fn departure(mut self, departure: DateTime<FixedOffset>) -> Self {
        self.departure = Some(departure);
        self
    }

    /// Search journeys arriving at this time (mutually exclusive with
    /// `departure` and pagination refs).
    pub fn arrival(mut self, arrival: DateTime<FixedOffset>) -> Self {
        self.arrival = Some(arrival);
        self
    }

    /// Pagination: journeys before the given `earlierRef`.
    pub fn earlier_than(mut self, earlier_ref: impl Into<String>) -> Self {
        self.earlier_than = Some(earlier_ref.into());
        self
    }

    /// Pagination: journeys after the given `laterRef`.
    pub fn later_than(mut self, later_ref: impl Into<String>) -> Self {
        self.later_than = Some(later_ref.into());
        self
    }

    /// Maximum number of journeys.
    pub fn results(mut self, results: i64) -> Self {
        self.results = Some(results);
        self
    }

    /// Include stopovers of each leg.
    pub fn stopovers(mut self, stopovers: bool) -> Self {
        self.stopovers = Some(stopovers);
        self
    }

    /// Maximum number of transfers.
    pub fn transfers(mut self, transfers: i64) -> Self {
        self.transfers = Some(transfers);
        self
    }

    /// Minimum transfer time in minutes.
    pub fn transfer_time(mut self, minutes: i64) -> Self {
        self.transfer_time = Some(minutes);
        self
    }

    /// Accessibility requirements (`partial` or `complete`).
    pub fn accessibility(mut self, accessibility: impl Into<String>) -> Self {
        self.accessibility = Some(accessibility.into());
        self
    }

    /// Only bike-friendly journeys?
    pub fn bike(mut self, bike: bool) -> Self {
        self.bike = Some(bike);
        self
    }

    /// Consider walking to nearby stations at the start?
    pub fn start_with_walking(mut self, yes: bool) -> Self {
        self.start_with_walking = Some(yes);
        self
    }

    /// Walking speed: `slow`, `normal` or `fast`.
    pub fn walking_speed(mut self, speed: impl Into<String>) -> Self {
        self.walking_speed = Some(speed.into());
        self
    }

    /// Return ticket information?
    ///
    /// On the DB instance this is only supported by
    /// [`TransportRestClient::refresh_journey`].
    pub fn tickets(mut self, tickets: bool) -> Self {
        self.tickets = Some(tickets);
        self
    }

    /// Return a shape for each leg?
    ///
    /// On the DB instance this is only supported by
    /// [`TransportRestClient::refresh_journey`].
    pub fn polylines(mut self, polylines: bool) -> Self {
        self.polylines = Some(polylines);
        self
    }

    /// Parse sub-stops of stations?
    pub fn sub_stops(mut self, sub_stops: bool) -> Self {
        self.sub_stops = Some(sub_stops);
        self
    }

    /// Parse entrances of stops/stations?
    pub fn entrances(mut self, entrances: bool) -> Self {
        self.entrances = Some(entrances);
        self
    }

    /// Include hints & warnings (default on).
    pub fn remarks(mut self, remarks: bool) -> Self {
        self.remarks = Some(remarks);
        self
    }

    /// Include the days each journey is served.
    pub fn scheduled_days(mut self, scheduled_days: bool) -> Self {
        self.scheduled_days = Some(scheduled_days);
        self
    }

    /// Also show journeys that are mathematically non-optimal (DB only).
    pub fn not_only_fast_routes(mut self, yes: bool) -> Self {
        self.not_only_fast_routes = Some(yes);
        self
    }

    /// Search for the lowest prices across the whole day (DB only).
    pub fn bestprice(mut self, yes: bool) -> Self {
        self.bestprice = Some(yes);
        self
    }

    /// Only connections usable with the Deutschlandticket (DB only).
    pub fn deutschland_ticket_connections_only(mut self, yes: bool) -> Self {
        self.deutschland_ticket_connections_only = Some(yes);
        self
    }

    /// Loyalty/discount card applied to prices (DB only), e.g.
    /// `"bahncard-2nd-50"`.
    pub fn loyalty_card(mut self, card: impl Into<String>) -> Self {
        self.loyalty_card = Some(card.into());
        self
    }

    /// Search first-class options (DB only).
    pub fn first_class(mut self, yes: bool) -> Self {
        self.first_class = Some(yes);
        self
    }

    /// Traveller age in years (DB only).
    pub fn age(mut self, age: i64) -> Self {
        self.age = Some(age);
        self
    }

    /// Traveller age group (DB only): `B`, `E`, `K`, `S` or `Y`.
    pub fn age_group(mut self, group: impl Into<String>) -> Self {
        self.age_group = Some(group.into());
        self
    }

    /// Routing mode (DB only). `REALTIME` is the default; use `HYBRID` for
    /// full pagination support and cancelled journeys.
    pub fn routing_mode(mut self, mode: impl Into<String>) -> Self {
        self.routing_mode = Some(mode.into());
        self
    }

    /// Filter included transport products.
    pub fn products<F: FnOnce(ProductSelection) -> ProductSelection>(mut self, f: F) -> Self {
        self.products = Some(f(ProductSelection::default()));
        self
    }

    /// Execute the request.
    pub async fn get(self) -> Result<JourneysResponse, TransportRestError> {
        // -- validation -----------------------------------------------------
        self.from.validate()?;
        self.to.validate()?;
        if let Some(via) = &self.via {
            via.validate()?;
        }
        if self.departure.is_some() && self.arrival.is_some() {
            return Err(TransportRestError::InvalidParameter(
                InvalidParameterError::other("departure and arrival are mutually exclusive"),
            ));
        }
        if (self.earlier_than.is_some() || self.later_than.is_some())
            && (self.departure.is_some() || self.arrival.is_some())
        {
            return Err(TransportRestError::InvalidParameter(
                InvalidParameterError::other(
                    "earlier_than/later_than cannot be combined with departure/arrival",
                ),
            ));
        }
        if self.earlier_than.is_some() && self.later_than.is_some() {
            return Err(TransportRestError::InvalidParameter(
                InvalidParameterError::other("earlier_than and later_than are mutually exclusive"),
            ));
        }
        if let Some(speed) = &self.walking_speed {
            if !matches!(speed.as_str(), "slow" | "normal" | "fast") {
                return Err(TransportRestError::InvalidParameter(
                    InvalidParameterError::new(
                        "walking_speed",
                        "must be one of slow, normal, fast",
                    ),
                ));
            }
        }
        if let Some(acc) = &self.accessibility {
            if !matches!(acc.as_str(), "partial" | "complete") {
                return Err(TransportRestError::InvalidParameter(
                    InvalidParameterError::new("accessibility", "must be 'partial' or 'complete'"),
                ));
            }
        }

        // -- serialization --------------------------------------------------
        let mut q = Query::new();
        self.from.encode("from", &mut q);
        self.to.encode("to", &mut q);
        if let Some(via) = &self.via {
            via.encode("via", &mut q);
        }
        if let Some(d) = self.departure {
            q.push(
                "departure",
                d.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            );
        }
        if let Some(a) = self.arrival {
            q.push(
                "arrival",
                a.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            );
        }
        q.opt("earlierThan", self.earlier_than);
        q.opt("laterThan", self.later_than);
        q.opt("results", self.results);
        q.opt("stopovers", self.stopovers);
        q.opt("transfers", self.transfers);
        q.opt("transferTime", self.transfer_time);
        q.opt("accessibility", self.accessibility);
        q.opt("bike", self.bike);
        q.opt("startWithWalking", self.start_with_walking);
        q.opt("walkingSpeed", self.walking_speed);
        q.opt("tickets", self.tickets);
        q.opt("polylines", self.polylines);
        q.opt("subStops", self.sub_stops);
        q.opt("entrances", self.entrances);
        q.opt("remarks", self.remarks);
        q.opt("scheduledDays", self.scheduled_days);
        q.opt("notOnlyFastRoutes", self.not_only_fast_routes);
        q.opt("bestprice", self.bestprice);
        q.opt(
            "deutschlandTicketConnectionsOnly",
            self.deutschland_ticket_connections_only,
        );
        q.opt("loyaltyCard", self.loyalty_card);
        q.opt("firstClass", self.first_class);
        q.opt("age", self.age);
        q.opt("ageGroup", self.age_group);
        q.opt("routingMode", self.routing_mode);
        if let Some(products) = self.products.filter(|p| !p.is_empty()) {
            products.encode(&mut q);
        }

        TransportRestClient { state: self.state }
            .get_json("/journeys", q)
            .await
    }
}

/// Builder for `GET /journeys/{ref}` – refresh a computed journey.
#[must_use = "builders do nothing until you call .get()"]
#[derive(Debug, Clone)]
pub struct RefreshJourneyBuilder {
    state: std::sync::Arc<ClientState>,
    refresh_token: String,
    stopovers: Option<bool>,
    tickets: Option<bool>,
    polylines: Option<bool>,
    sub_stops: Option<bool>,
    entrances: Option<bool>,
    remarks: Option<bool>,
    scheduled_days: Option<bool>,
    not_only_fast_routes: Option<bool>,
    bestprice: Option<bool>,
    language: Option<String>,
}

impl RefreshJourneyBuilder {
    pub(crate) fn new(state: std::sync::Arc<ClientState>, refresh_token: String) -> Self {
        Self {
            state,
            refresh_token,
            stopovers: None,
            tickets: None,
            polylines: None,
            sub_stops: None,
            entrances: None,
            remarks: None,
            scheduled_days: None,
            not_only_fast_routes: None,
            bestprice: None,
            language: None,
        }
    }

    /// Include stopovers of each leg.
    pub fn stopovers(mut self, stopovers: bool) -> Self {
        self.stopovers = Some(stopovers);
        self
    }

    /// Return ticket information (mutually exclusive with `polylines`).
    pub fn tickets(mut self, tickets: bool) -> Self {
        self.tickets = Some(tickets);
        self
    }

    /// Return leg shapes (mutually exclusive with `tickets`).
    pub fn polylines(mut self, polylines: bool) -> Self {
        self.polylines = Some(polylines);
        self
    }

    /// Parse sub-stops of stations?
    pub fn sub_stops(mut self, sub_stops: bool) -> Self {
        self.sub_stops = Some(sub_stops);
        self
    }

    /// Parse entrances of stops/stations?
    pub fn entrances(mut self, entrances: bool) -> Self {
        self.entrances = Some(entrances);
        self
    }

    /// Include hints & warnings (default on).
    pub fn remarks(mut self, remarks: bool) -> Self {
        self.remarks = Some(remarks);
        self
    }

    /// Include the days the journey is served.
    pub fn scheduled_days(mut self, scheduled_days: bool) -> Self {
        self.scheduled_days = Some(scheduled_days);
        self
    }

    /// Also show non-optimal journeys (DB only).
    pub fn not_only_fast_routes(mut self, yes: bool) -> Self {
        self.not_only_fast_routes = Some(yes);
        self
    }

    /// Search lowest prices across the day (DB only).
    pub fn bestprice(mut self, yes: bool) -> Self {
        self.bestprice = Some(yes);
        self
    }

    /// Language of result texts.
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Execute the request.
    pub async fn get(self) -> Result<JourneyResponse, TransportRestError> {
        if self.refresh_token.trim().is_empty() {
            return Err(TransportRestError::InvalidParameter(
                InvalidParameterError::new("refresh_token", "must not be empty"),
            ));
        }
        if self.tickets.unwrap_or(false) && self.polylines.unwrap_or(false) {
            return Err(TransportRestError::InvalidParameter(
                InvalidParameterError::other("tickets and polylines are mutually exclusive"),
            ));
        }
        let mut q = Query::new();
        q.opt("stopovers", self.stopovers);
        q.opt("tickets", self.tickets);
        q.opt("polylines", self.polylines);
        q.opt("subStops", self.sub_stops);
        q.opt("entrances", self.entrances);
        q.opt("remarks", self.remarks);
        q.opt("scheduledDays", self.scheduled_days);
        q.opt("notOnlyFastRoutes", self.not_only_fast_routes);
        q.opt("bestprice", self.bestprice);
        q.opt("language", self.language);

        let path = format!("/journeys/{}", encode_path_segment(&self.refresh_token));
        TransportRestClient { state: self.state }
            .get_json(&path, q)
            .await
    }
}
