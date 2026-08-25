"""Endpoint builders mirroring the Rust core API."""

from __future__ import annotations

import datetime as _dt
from typing import Any, Callable, Optional

from . import errors as _e
from .models_gen import (
    ArrivalsResponse,
    DeparturesResponse,
    JourneyResponse,
    JourneysResponse,
    RadarResponse,
    ReachableFromResponse,
    Station,
    TripResponse,
    TripsResponse,
    parse_LocationResult,
    parse_StopOrStation,
)


def _iso(when: _dt.datetime) -> str:
    if when.tzinfo is None:
        raise _e.InvalidParameterError("when", "datetime must carry a UTC offset")
    return when.isoformat(timespec="seconds")


def _require(condition, parameter, reason) -> None:
    if not condition:
        raise _e.InvalidParameterError(parameter, reason)


def _bool(value: bool) -> str:
    return "true" if value else "false"


def _locations_model(data):
    return [parse_LocationResult(i) if isinstance(i, dict) else i for i in data]


def _stations_model(data):
    return [Station.from_dict(i) if isinstance(i, dict) else i for i in data]


class JourneyPlace:
    """Factory helpers for journey ``from``/``to``/``via`` places."""

    @staticmethod
    def stop_id(stop_id: str) -> dict:
        return {"form": "id", "id": stop_id}

    @staticmethod
    def name(name: str) -> dict:
        return {"form": "name", "name": name}

    @staticmethod
    def poi(poi_id: str, latitude: float, longitude: float) -> dict:
        return {"form": "poi", "id": poi_id, "latitude": latitude, "longitude": longitude}

    @staticmethod
    def address(latitude: float, longitude: float, address: str) -> dict:
        return {
            "form": "address",
            "latitude": latitude,
            "longitude": longitude,
            "address": address,
        }


def _encode_place(prefix: str, place: Optional[dict], params: list) -> None:
    if place is None:
        return
    form = place["form"]
    if form == "id":
        params.append((prefix, place["id"]))
    elif form == "name":
        params.append((prefix + ".name", place["name"]))
    elif form == "poi":
        params.append((prefix + ".id", place["id"]))
        params.append((prefix + ".latitude", repr(place["latitude"])))
        params.append((prefix + ".longitude", repr(place["longitude"])))
    elif form == "address":
        params.append((prefix + ".latitude", repr(place["latitude"])))
        params.append((prefix + ".longitude", repr(place["longitude"])))
        params.append((prefix + ".address", place["address"]))
    else:
        raise _e.InvalidParameterError(None, f"unknown JourneyPlace form {form!r}")


class ProductSelection:
    """Product filter; unset keys are omitted (server default includes all)."""

    def __init__(self) -> None:
        self._entries = []

    def set(self, key: str, enabled: bool) -> "ProductSelection":
        for i, (k, _) in enumerate(self._entries):
            if k == key:
                self._entries[i] = (k, enabled)
                break
        else:
            self._entries.append((key, enabled))
        return self

    bus = suburban = subway = tram = ferry = express = regional = None  # set below

    def national_express(self, enabled: bool) -> "ProductSelection":
        return self.set("nationalExpress", enabled)

    def national(self, enabled: bool) -> "ProductSelection":
        return self.set("national", enabled)

    def regional_express(self, enabled: bool) -> "ProductSelection":
        return self.set("regionalExpress", enabled)

    def bus(self, enabled: bool) -> "ProductSelection":
        return self.set("bus", enabled)

    def suburban(self, enabled: bool) -> "ProductSelection":
        return self.set("suburban", enabled)

    def subway(self, enabled: bool) -> "ProductSelection":
        return self.set("subway", enabled)

    def tram(self, enabled: bool) -> "ProductSelection":
        return self.set("tram", enabled)

    def ferry(self, enabled: bool) -> "ProductSelection":
        return self.set("ferry", enabled)

    def taxi(self, enabled: bool) -> "ProductSelection":
        return self.set("taxi", enabled)

    def express(self, enabled: bool) -> "ProductSelection":
        return self.set("express", enabled)

    def regional(self, enabled: bool) -> "ProductSelection":
        return self.set("regional", enabled)

    def encode(self, params: list) -> None:
        for key, enabled in self._entries:
            params.append((key, _bool(enabled)))


class _BoardBase:
    """Shared state of departure/arrival board builders."""

    def __init__(self) -> None:
        self.when_ = None
        self.direction_ = None
        self.duration_ = None
        self.results_ = None
        self.stopovers_ = None
        self.include_related_stations_ = None
        self.lines_of_stops_ = None
        self.remarks_ = None
        self.language_ = None
        self.more_stops_ = None
        self.products_ = None

    def encode(self, params: list) -> None:
        if self.when_ is not None:
            params.append(("when", _iso(self.when_)))
        if self.direction_ is not None:
            params.append(("direction", self.direction_))
        if self.duration_ is not None:
            params.append(("duration", str(self.duration_)))
        if self.results_ is not None:
            params.append(("results", str(self.results_)))
        if self.stopovers_ is not None:
            params.append(("stopovers", _bool(self.stopovers_)))
        if self.include_related_stations_ is not None:
            params.append(("includeRelatedStations", _bool(self.include_related_stations_)))
        if self.lines_of_stops_ is not None:
            params.append(("linesOfStops", _bool(self.lines_of_stops_)))
        if self.remarks_ is not None:
            params.append(("remarks", _bool(self.remarks_)))
        if self.language_ is not None:
            params.append(("language", self.language_))
        if self.more_stops_:
            params.append(("moreStops", ",".join(self.more_stops_)))
        if self.products_ is not None and self.products_._entries:
            self.products_.encode(params)


class _Builder:
    def __init__(self, client, path: str, model, capability: Optional[str] = None) -> None:
        self._client = client
        self._path = path
        self._model = model
        self._capability = capability
        self._params: list = []

    def _opt(self, key: str, value) -> None:
        if value is not None:
            self._params.append((key, value))

    def get(self):
        return self._client.get_json(self._path, self._params, self._model, self._capability)

    async def get_async(self):
        return await self._client.get_json_async(
            self._path, self._params, self._model, self._capability
        )


class LocationsBuilder(_Builder):
    """``GET /locations`` – search stops, POIs and addresses."""

    def __init__(self, client) -> None:
        super().__init__(client, "/locations", _locations_model)
        self._query = None

    def query(self, q: str) -> "LocationsBuilder":
        _require(bool(q and q.strip()), "query", "a non-empty search term is required")
        self._query = q
        return self

    def fuzzy(self, value: bool) -> "LocationsBuilder":
        self._opt("fuzzy", _bool(value))
        return self

    def results(self, n: int) -> "LocationsBuilder":
        self._opt("results", n)
        return self

    def stops(self, value: bool) -> "LocationsBuilder":
        self._opt("stops", _bool(value))
        return self

    def addresses(self, value: bool) -> "LocationsBuilder":
        self._opt("addresses", _bool(value))
        return self

    def poi(self, value: bool) -> "LocationsBuilder":
        self._opt("poi", _bool(value))
        return self

    def lines_of_stops(self, value: bool) -> "LocationsBuilder":
        self._opt("linesOfStops", _bool(value))
        return self

    def language(self, language: str) -> "LocationsBuilder":
        self._opt("language", language)
        return self

    def get(self):
        _require(self._query is not None, "query", "query() is required")
        self._params.insert(0, ("query", self._query))
        return super().get()


class NearbyBuilder(_Builder):
    """``GET /locations/nearby``."""

    def __init__(self, client) -> None:
        super().__init__(client, "/locations/nearby", _locations_model)
        self._latitude = None
        self._longitude = None

    def latitude(self, value: float) -> "NearbyBuilder":
        _require(-90.0 <= value <= 90.0, "latitude", "must be within [-90, 90]")
        self._latitude = value
        return self

    def longitude(self, value: float) -> "NearbyBuilder":
        _require(-180.0 <= value <= 180.0, "longitude", "must be within [-180, 180]")
        self._longitude = value
        return self

    def results(self, n: int) -> "NearbyBuilder":
        self._opt("results", n)
        return self

    def distance(self, meters: int) -> "NearbyBuilder":
        self._opt("distance", meters)
        return self

    def stops(self, value: bool) -> "NearbyBuilder":
        self._opt("stops", _bool(value))
        return self

    def poi(self, value: bool) -> "NearbyBuilder":
        self._opt("poi", _bool(value))
        return self

    def language(self, language: str) -> "NearbyBuilder":
        self._opt("language", language)
        return self

    def get(self):
        _require(
            self._latitude is not None and self._longitude is not None,
            None,
            "latitude and longitude are both required",
        )
        self._params.insert(0, ("longitude", repr(self._longitude)))
        self._params.insert(0, ("latitude", repr(self._latitude)))
        return super().get()


class StopBuilder(_Builder):
    """``GET /stops/{id}``."""

    def __init__(self, client, stop_id: str) -> None:
        from .client import _encode_path_segment

        super().__init__(
            client, f"/stops/{_encode_path_segment(stop_id)}", parse_StopOrStation
        )
        _require(stop_id and stop_id.strip(), "stop_id", "must not be empty")

    def lines_of_stops(self, value: bool) -> "StopBuilder":
        self._opt("linesOfStops", _bool(value))
        return self

    def language(self, language: str) -> "StopBuilder":
        self._opt("language", language)
        return self


class _Board(_Builder):
    @classmethod
    def create(cls, client, stop_id: str) -> "_Board":
        from .client import _encode_path_segment

        _require(stop_id and stop_id.strip(), "stop_id", "must not be empty")
        model = cls._MODEL
        return cls(client, f"/stops/{_encode_path_segment(stop_id)}/" + cls._SUFFIX, model)

    def __init__(self, client, path, model) -> None:
        super().__init__(client, path, model)
        self.base = _BoardBase()

    # -- shared board options -------------------------------------------------
    def when(self, when: _dt.datetime) -> "_Board":
        self.base.when_ = when
        return self

    def direction(self, direction: str) -> "_Board":
        self.base.direction_ = direction
        return self

    def duration(self, minutes: int) -> "_Board":
        self.base.duration_ = minutes
        return self

    def results(self, results: int) -> "_Board":
        self.base.results_ = results
        return self

    def stopovers(self, value: bool) -> "_Board":
        self.base.stopovers_ = value
        return self

    def include_related_stations(self, value: bool) -> "_Board":
        self.base.include_related_stations_ = value
        return self

    def lines_of_stops(self, value: bool) -> "_Board":
        self.base.lines_of_stops_ = value
        return self

    def remarks(self, value: bool) -> "_Board":
        self.base.remarks_ = value
        return self

    def language(self, language: str) -> "_Board":
        self.base.language_ = language
        return self

    def more_stops(self, stop_ids: Iterable) -> "_Board":
        self.base.more_stops_ = list(stop_ids)
        return self

    def products(self, configure: Callable) -> "_Board":
        selection = ProductSelection()
        self.base.products_ = configure(selection)
        return self

    def get(self):
        self.base.encode(self._params)
        return super().get()


class DeparturesBuilder(_Board):
    """``GET /stops/{id}/departures``."""

    _SUFFIX = "departures"
    _MODEL = staticmethod(DeparturesResponse.from_dict)


class ArrivalsBuilder(_Board):
    """``GET /stops/{id}/arrivals``."""

    _SUFFIX = "arrivals"
    _MODEL = staticmethod(ArrivalsResponse.from_dict)


class JourneysBuilder(_Builder):
    """``GET /journeys`` – route search."""

    def __init__(self, client, frm: dict, to: dict) -> None:
        super().__init__(client, "/journeys", JourneysResponse.from_dict)
        self._from = frm
        self._to = to
        self._via = None
        self._departure = None
        self._arrival = None
        self._earlier_than = None
        self._later_than = None
        self._options: list = []
        self._products = None

    def via(self, place: dict) -> "JourneysBuilder":
        self._via = place
        return self

    def departure(self, when: _dt.datetime) -> "JourneysBuilder":
        self._departure = when
        return self

    def arrival(self, when: _dt.datetime) -> "JourneysBuilder":
        self._arrival = when
        return self

    def earlier_than(self, ref: str) -> "JourneysBuilder":
        self._earlier_than = ref
        return self

    def later_than(self, ref: str) -> "JourneysBuilder":
        self._later_than = ref
        return self

    # -- simple option setters -----------------------------------------------
    def results(self, n: int) -> "JourneysBuilder":
        self._options.append(("results", str(n)))
        return self

    def stopovers(self, value: bool) -> "JourneysBuilder":
        self._options.append(("stopovers", _bool(value)))
        return self

    def transfers(self, n: int) -> "JourneysBuilder":
        self._options.append(("transfers", str(n)))
        return self

    def transfer_time(self, minutes: int) -> "JourneysBuilder":
        self._options.append(("transferTime", str(minutes)))
        return self

    def accessibility(self, value: str) -> "JourneysBuilder":
        _require(value in ("partial", "complete"), "accessibility", "must be 'partial' or 'complete'")
        self._options.append(("accessibility", value))
        return self

    def bike(self, value: bool) -> "JourneysBuilder":
        self._options.append(("bike", _bool(value)))
        return self

    def start_with_walking(self, value: bool) -> "JourneysBuilder":
        self._options.append(("startWithWalking", _bool(value)))
        return self

    def walking_speed(self, value: str) -> "JourneysBuilder":
        _require(value in ("slow", "normal", "fast"), "walking_speed", "must be slow|normal|fast")
        self._options.append(("walkingSpeed", value))
        return self

    def tickets(self, value: bool) -> "JourneysBuilder":
        self._options.append(("tickets", _bool(value)))
        return self

    def polylines(self, value: bool) -> "JourneysBuilder":
        self._options.append(("polylines", _bool(value)))
        return self

    def remarks(self, value: bool) -> "JourneysBuilder":
        self._options.append(("remarks", _bool(value)))
        return self

    def scheduled_days(self, value: bool) -> "JourneysBuilder":
        self._options.append(("scheduledDays", _bool(value)))
        return self

    def not_only_fast_routes(self, value: bool) -> "JourneysBuilder":
        self._options.append(("notOnlyFastRoutes", _bool(value)))
        return self

    def bestprice(self, value: bool) -> "JourneysBuilder":
        self._options.append(("bestprice", _bool(value)))
        return self

    def loyalty_card(self, card: str) -> "JourneysBuilder":
        self._options.append(("loyaltyCard", card))
        return self

    def first_class(self, value: bool) -> "JourneysBuilder":
        self._options.append(("firstClass", _bool(value)))
        return self

    def routing_mode(self, mode: str) -> "JourneysBuilder":
        self._options.append(("routingMode", mode))
        return self

    def products(self, configure: Callable) -> "JourneysBuilder":
        self._products = configure(ProductSelection())
        return self

    def get(self):
        _validate_place("from", self._from)
        _validate_place("to", self._to)
        if self._via is not None:
            _validate_place("via", self._via)
        if self._departure is not None and self._arrival is not None:
            raise _e.InvalidParameterError(None, "departure and arrival are mutually exclusive")
        if (self._earlier_than or self._later_than) and (self._departure or self._arrival):
            raise _e.InvalidParameterError(
                None, "earlier_than/later_than cannot be combined with departure/arrival"
            )
        if self._earlier_than and self._later_than:
            raise _e.InvalidParameterError(None, "earlier_than and later_than are mutually exclusive")

        params: list = []
        _encode_place("from", self._from, params)
        _encode_place("to", self._to, params)
        if self._via is not None:
            _encode_place("via", self._via, params)
        if self._departure is not None:
            params.append(("departure", _iso(self._departure)))
        if self._arrival is not None:
            params.append(("arrival", _iso(self._arrival)))
        if self._earlier_than:
            params.append(("earlierThan", self._earlier_than))
        if self._later_than:
            params.append(("laterThan", self._later_than))
        params.extend(self._options)
        if self._products is not None and self._products._entries:
            self._products.encode(params)

        self._params = params
        return super().get()


def _validate_place(param: str, place: Optional[dict]) -> None:
    _require(place is not None, param, f"{param} is required")
    form = place["form"]
    if form == "poi":
        _require(bool(place.get("id")), param + ".id", "POI id must not be empty")
    elif form == "address":
        _require(bool(place.get("address")), param + ".address", "address must not be empty")


class RefreshJourneyBuilder(_Builder):
    """``GET /journeys/{ref}`` – refresh a computed journey."""

    def __init__(self, client, refresh_token: str) -> None:
        from .client import _encode_path_segment

        _require(refresh_token and refresh_token.strip(), "refresh_token", "must not be empty")
        super().__init__(
            client,
            f"/journeys/{_encode_path_segment(refresh_token)}",
            JourneyResponse.from_dict,
        )
        self._tickets = None
        self._polylines = None

    def stopovers(self, value: bool) -> "RefreshJourneyBuilder":
        self._opt("stopovers", _bool(value))
        return self

    def tickets(self, value: bool) -> "RefreshJourneyBuilder":
        self._tickets = value
        return self

    def polylines(self, value: bool) -> "RefreshJourneyBuilder":
        self._polylines = value
        return self

    def remarks(self, value: bool) -> "RefreshJourneyBuilder":
        self._opt("remarks", _bool(value))
        return self

    def language(self, language: str) -> "RefreshJourneyBuilder":
        self._opt("language", language)
        return self

    def get(self):
        if self._tickets and self._polylines:
            raise _e.InvalidParameterError(None, "tickets and polylines are mutually exclusive")
        return super().get()


class TripBuilder(_Builder):
    """``GET /trips/{id}``."""

    def __init__(self, client, trip_id: str) -> None:
        from .client import _encode_path_segment

        _require(trip_id and trip_id.strip(), "trip_id", "must not be empty")
        super().__init__(client, f"/trips/{_encode_path_segment(trip_id)}", TripResponse.from_dict)

    def stopovers(self, value: bool) -> "TripBuilder":
        self._opt("stopovers", _bool(value))
        return self

    def remarks(self, value: bool) -> "TripBuilder":
        self._opt("remarks", _bool(value))
        return self

    def polyline(self, value: bool) -> "TripBuilder":
        self._opt("polyline", _bool(value))
        return self

    def language(self, language: str) -> "TripBuilder":
        self._opt("language", language)
        return self


class TripsByNameBuilder(_Builder):
    """``GET /trips`` – find trips by name (capability ``trips_by_name``)."""

    def __init__(self, client, query: str) -> None:
        super().__init__(
            client, "/trips", TripsResponse.from_dict, capability="trips_by_name"
        )
        self._opt("query", query or "*")

    def only_currently_running(self, value: bool) -> "TripsByNameBuilder":
        self._opt("onlyCurrentlyRunning", _bool(value))
        return self

    def line_name(self, name: str) -> "TripsByNameBuilder":
        self._opt("lineName", name)
        return self


class RadarBuilder(_Builder):
    """``GET /radar`` (capability ``radar``)."""

    def __init__(self, client) -> None:
        super().__init__(client, "/radar", RadarResponse.from_dict, capability="radar")
        self._box = {}

    def north(self, v: float) -> "RadarBuilder":
        self._box["north"] = v
        return self

    def west(self, v: float) -> "RadarBuilder":
        self._box["west"] = v
        return self

    def south(self, v: float) -> "RadarBuilder":
        self._box["south"] = v
        return self

    def east(self, v: float) -> "RadarBuilder":
        self._box["east"] = v
        return self

    def results(self, n: int) -> "RadarBuilder":
        self._opt("results", n)
        return self

    def frames(self, n: int) -> "RadarBuilder":
        self._opt("frames", n)
        return self

    def duration(self, seconds: int) -> "RadarBuilder":
        self._opt("duration", seconds)
        return self

    def get(self):
        _require(
            len(self._box) == 4,
            None,
            "north, west, south and east are all required",
        )
        _require(
            self._box["south"] <= self._box["north"] and self._box["west"] <= self._box["east"],
            None,
            "bounding box is invalid: require south <= north and west <= east",
        )
        ordered = [("north", "north"), ("west", "west"), ("south", "south"), ("east", "east")]
        for key, name in ordered:
            self._params.insert(len(ordered), (key, repr(self._box[name])))
        return super().get()


class ReachableFromBuilder(_Builder):
    """``GET /stops/reachable-from`` (capability ``reachable_from``)."""

    def __init__(self, client) -> None:
        super().__init__(
            client,
            "/stops/reachable-from",
            ReachableFromResponse.from_dict,
            capability="reachable_from",
        )
        self._latitude = None
        self._longitude = None

    def latitude(self, v: float) -> "ReachableFromBuilder":
        self._latitude = v
        return self

    def longitude(self, v: float) -> "ReachableFromBuilder":
        self._longitude = v
        return self

    def max_transfers(self, n: int) -> "ReachableFromBuilder":
        self._opt("maxTransfers", n)
        return self

    def max_duration(self, minutes: int) -> "ReachableFromBuilder":
        self._opt("maxDuration", minutes)
        return self

    def get(self):
        _require(
            self._latitude is not None and self._longitude is not None,
            None,
            "latitude and longitude are both required",
        )
        self._params.insert(0, ("longitude", repr(self._longitude)))
        self._params.insert(0, ("latitude", repr(self._latitude)))
        return super().get()


class StationsBuilder(_Builder):
    """``GET /stations`` – static station directory (capability ``stations``)."""

    def __init__(self, client) -> None:
        super().__init__(client, "/stations", _stations_model, capability="stations")

    def query(self, q: str) -> "StationsBuilder":
        self._opt("query", q)
        return self

    def results(self, n: int) -> "StationsBuilder":
        self._opt("results", n)
        return self


class StationBuilder(_Builder):
    """``GET /stations/{id}`` (capability ``stations``)."""

    def __init__(self, client, station_id: str) -> None:
        from .client import _encode_path_segment

        _require(station_id and station_id.strip(), "station_id", "must not be empty")
        super().__init__(
            client,
            f"/stations/{_encode_path_segment(station_id)}",
            Station.from_dict,
            capability="stations",
        )


class StopsSearchBuilder(_Builder):
    """``GET /stops`` – static stop search (BVG/VBB; capability ``stops_search``)."""

    def __init__(self, client) -> None:
        super().__init__(
            client, "/stops", _locations_model, capability="stops_search"
        )

    def query(self, q: str) -> "StopsSearchBuilder":
        self._opt("query", q)
        return self

    def limit(self, n: int) -> "StopsSearchBuilder":
        self._opt("limit", n)
        return self
