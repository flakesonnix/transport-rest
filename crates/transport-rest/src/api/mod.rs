//! Endpoint builders grouped by resource.
//!
//! Each accessor on [`TransportRestClient`](crate::TransportRestClient)
//! returns a builder; calling `.get().await?` executes the request.

pub mod departures;
pub mod journeys;
pub mod locations;
pub mod nearby;
pub mod radar;
pub mod reachable_from;
pub mod stations;
pub mod stops;
pub mod trips;

use crate::TransportRestClient;

impl TransportRestClient {
    /// Search stops/stations, POIs and addresses by name.
    pub fn locations(&self) -> locations::LocationsBuilder {
        locations::LocationsBuilder::new(self.state.clone())
    }

    /// Find stops/stations & POIs close to a geolocation.
    pub fn nearby(&self) -> nearby::NearbyBuilder {
        nearby::NearbyBuilder::new(self.state.clone())
    }

    /// Fetch a single stop/station by ID.
    pub fn stop(&self, id: impl Into<String>) -> stops::StopBuilder {
        stops::StopBuilder::new(self.state.clone(), id.into())
    }

    /// Departure board of a stop/station.
    pub fn departures(&self, stop_id: impl Into<String>) -> departures::DeparturesBuilder {
        departures::DeparturesBuilder::departures(self.state.clone(), stop_id.into())
    }

    /// Arrival board of a stop/station.
    pub fn arrivals(&self, stop_id: impl Into<String>) -> departures::ArrivalsBuilder {
        departures::ArrivalsBuilder::arrivals(self.state.clone(), stop_id.into())
    }

    /// Find journeys from A to B.
    pub fn journeys(
        &self,
        from: impl Into<crate::request::JourneyPlace>,
        to: impl Into<crate::request::JourneyPlace>,
    ) -> journeys::JourneysBuilder {
        journeys::JourneysBuilder::new(self.state.clone(), from.into(), to.into())
    }

    /// Refresh a previously computed journey by its `refreshToken`.
    pub fn refresh_journey(&self, refresh_token: impl Into<String>) -> journeys::RefreshJourneyBuilder {
        journeys::RefreshJourneyBuilder::new(self.state.clone(), refresh_token.into())
    }

    /// Fetch a trip by ID.
    pub fn trip(&self, id: impl Into<String>) -> trips::TripBuilder {
        trips::TripBuilder::new(self.state.clone(), id.into())
    }

    /// Find vehicles moving inside a bounding box (capability-gated).
    pub fn radar(&self) -> radar::RadarBuilder {
        radar::RadarBuilder::new(self.state.clone())
    }

    /// Find stops reachable within a time budget (capability-gated).
    pub fn reachable_from(&self) -> reachable_from::ReachableFromBuilder {
        reachable_from::ReachableFromBuilder::new(self.state.clone())
    }

    /// Find trips by name (capability-gated).
    pub fn trips_by_name(&self, query: impl Into<String>) -> trips::TripsByNameBuilder {
        trips::TripsByNameBuilder::new(self.state.clone(), query.into())
    }

    /// Search the static station directory (DB instance).
    pub fn stations(&self) -> stations::StationsBuilder {
        stations::StationsBuilder::new(self.state.clone())
    }

    /// Get one station from the static directory (DB instance).
    pub fn station(&self, id: impl Into<String>) -> stations::StationBuilder {
        stations::StationBuilder::new(self.state.clone(), id.into())
    }

    /// Search stops by name against the static dataset (BVG/VBB instances).
    pub fn stops_search(&self) -> stations::StopsSearchBuilder {
        stations::StopsSearchBuilder::new(self.state.clone())
    }
}
