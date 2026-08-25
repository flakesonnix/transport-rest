//! Typed async client for the [transport.rest](https://transport.rest) transit APIs.
//!
//! transport.rest is a family of REST instances covering different German and
//! European transit providers (Deutsche Bahn, BVG Berlin, VBB, Poland, ...).
//! This crate offers one consistent, strongly typed API surface over all of
//! them, with per-provider capabilities.
//!
//! # Quick start
//!
//! ```no_run
//! use transport_rest::TransportRestClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), transport_rest::TransportRestError> {
//!     let client = TransportRestClient::new();
//!
//!     let locations = client
//!         .locations()
//!         .query("Berlin")
//!         .results(5)
//!         .get()
//!         .await?;
//!
//!     for location in &locations {
//!         println!("{}", location.name().unwrap_or("<unnamed>"));
//!     }
//!     Ok(())
//! }
//! ```
//!
//! # Design notes
//!
//! * All endpoint accessors return builders; `.get().await?` executes.
//! * Unknown JSON fields are ignored, open enums fall back to an `Other`
//!   variant: the library stays compatible when the upstream API evolves.
//! * Errors are structured ([`TransportRestError`]) and never panic.
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::unwrap_used, clippy::expect_used)]

pub mod api;
pub mod error;

pub(crate) mod client;
mod products;
pub(crate) mod request;

mod builder;
pub mod models;

mod util;

pub use error::{
    ApiError, CapabilityNotSupportedError, HttpError, InvalidParameterError, NetworkError,
    RateLimitedError, Result, SerializationError, SerializationErrorKind, TimeoutError, TimeoutKind,
    TransportRestError,
};

pub use api::departures::{ArrivalsBuilder, DeparturesBuilder};
pub use api::journeys::{JourneysBuilder, RefreshJourneyBuilder};
pub use api::locations::LocationsBuilder;
pub use api::nearby::NearbyBuilder;
pub use api::radar::RadarBuilder;
pub use api::reachable_from::ReachableFromBuilder;
pub use api::stations::{StationBuilder, StationsBuilder, StopsSearchBuilder};
pub use api::stops::StopBuilder;
pub use api::trips::{TripBuilder, TripsByNameBuilder};
pub use builder::TransportRestClientBuilder;
pub use products::ProductSelection;
pub use request::JourneyPlace;

/// Re-exported [`chrono`] types used by model fields.
pub mod datetime {
    pub use chrono::DateTime;
    pub use chrono::FixedOffset;
}

use std::sync::Arc;

/// A transport.rest instance ("provider") with its own region and capabilities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provider {
    /// Deutsche Bahn, <https://v6.db.transport.rest>.
    Db,
    /// BVG Berlin & Brandenburg, <https://v6.bvg.transport.rest>.
    Bvg,
    /// VBB Berlin & Brandenburg, <https://v6.vbb.transport.rest>.
    Vbb,
    /// Poland, <https://poland.transport.rest>.
    Poland,
    /// Any other hafas-rest-api/db-rest compatible instance.
    Custom {
        /// Base URL of the instance, e.g. `https://my-instance.example.org`.
        base_url: String,
    },
}

impl Provider {
    /// Default base URL of this provider, if it has one.
    pub fn default_base_url(&self) -> Option<&'static str> {
        match self {
            Provider::Db => Some("https://v6.db.transport.rest"),
            Provider::Bvg => Some("https://v6.bvg.transport.rest"),
            Provider::Vbb => Some("https://v6.vbb.transport.rest"),
            Provider::Poland => Some("https://poland.transport.rest"),
            Provider::Custom { .. } => None,
        }
    }

    /// Capabilities known to be supported by this provider's public instance.
    pub(crate) fn default_capabilities(&self) -> Capabilities {
        let mut caps = Capabilities::core();
        match self {
            // db-vendo has no radar/reachable-from/trips-by-name, but ships
            // a static station directory.
            Provider::Db => caps.insert(Capability::Stations),
            // HAFAS based instances support live tracking endpoints.
            Provider::Bvg | Provider::Vbb | Provider::Poland => {
                caps.insert(Capability::Radar);
                caps.insert(Capability::ReachableFrom);
                caps.insert(Capability::TripsByName);
                if matches!(self, Provider::Bvg | Provider::Vbb) {
                    caps.insert(Capability::StopsSearch);
                }
            }
            // Unknown instance: be conservative.
            Provider::Custom { .. } => {}
        }
        caps
    }
}

/// An optional endpoint group not present on every provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Capability {
    /// `GET /radar`: vehicles moving inside a bounding box.
    Radar,
    /// `GET /stops/reachable-from`: isochrone-ish reachability.
    ReachableFrom,
    /// `GET /trips`: find trips by name.
    TripsByName,
    /// `GET /stations`, `GET /stations/{id}`: static station directory (DB).
    Stations,
    /// `GET /stops`: static stop name search (BVG/VBB).
    StopsSearch,
}

impl Capability {
    /// Human readable capability id as used in docs/API_ANALYSIS.md.
    pub fn as_str(&self) -> &'static str {
        match self {
            Capability::Radar => "radar",
            Capability::ReachableFrom => "reachable_from",
            Capability::TripsByName => "trips_by_name",
            Capability::Stations => "stations",
            Capability::StopsSearch => "stops_search",
        }
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Bitset of supported endpoint groups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Capabilities(u16);

impl Capabilities {
    fn core() -> Self {
        Self(0)
    }

    fn insert(&mut self, cap: Capability) {
        self.0 |= cap.bit();
    }

    pub(crate) fn contains(&self, cap: Capability) -> bool {
        self.0 & cap.bit() != 0
    }

    pub(crate) fn insert_raw(&mut self, cap: Capability) {
        self.insert(cap);
    }
}

impl Capability {
    fn bit(self) -> u16 {
        1 << (self as u16)
    }
}

/// Shared state of a [`TransportRestClient`].
#[derive(Debug)]
pub(crate) struct ClientState {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: url::Url,
    pub(crate) provider: Provider,
    pub(crate) capabilities: Capabilities,
    pub(crate) max_response_bytes: usize,
}

/// Client for a transport.rest instance.
///
/// Cheap to clone; connection pools are shared between clones.
#[derive(Clone)]
pub struct TransportRestClient {
    pub(crate) state: Arc<ClientState>,
}

impl std::fmt::Debug for TransportRestClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransportRestClient")
            .field("provider", &self.state.provider)
            .field("base_url", &self.state.base_url.as_str())
            .finish_non_exhaustive()
    }
}

impl TransportRestClient {
    /// Create a client for the Deutsche Bahn instance
    /// (`v6.db.transport.rest`) with sensible defaults:
    ///
    /// * 30s overall request timeout, 10s connect timeout
    /// * TLS via rustls, HTTP/2, gzip/brotli decompression
    /// * response size limit of 16 MiB
    ///
    /// # Panics
    ///
    /// Panics only if the TLS backend cannot be initialized, which in practice
    /// never happens on supported platforms. Use
    /// [`TransportRestClientBuilder::build`] if you need fallibility.
    #[allow(clippy::expect_used)]
    pub fn new() -> Self {
        Self::builder().build().expect("default TLS backend failed to initialize")
    }

    /// Start configuring a client.
    pub fn builder() -> TransportRestClientBuilder {
        TransportRestClientBuilder::default()
    }
}

impl TransportRestClient {
    /// Base URL all requests are resolved against.
    pub fn base_url(&self) -> &url::Url {
        &self.state.base_url
    }

    /// The configured provider.
    pub fn provider(&self) -> &Provider {
        &self.state.provider
    }

    /// True if this client may use the given endpoint group.
    pub fn supports(&self, capability: Capability) -> bool {
        self.state.capabilities.contains(capability)
    }
}

impl Default for TransportRestClient {
    fn default() -> Self {
        Self::new()
    }
}
