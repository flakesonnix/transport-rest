"""Binding tests: create client, request, deserialize, errors, async.

Runs fully offline against a local mock server (stdlib only).
"""

from __future__ import annotations

import asyncio
import datetime as dt
import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[0]))
from mock_server import MockServer  # noqa: E402

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from transport_rest import TransportRestClient, JourneyPlace, ProductSelection  # noqa: E402
from transport_rest import errors as e  # noqa: E402

WHEN = dt.datetime.fromisoformat("2026-08-01T12:00:00+02:00")


class TransportRestTestCase(unittest.TestCase):
    def setUp(self):
        self.server = MockServer().__enter__()
        self.client = TransportRestClient(base_url=self.server.url)

    def tearDown(self):
        self.server.__exit__(None, None, None)


class TestLocations(TransportRestTestCase):
    def test_locations_roundtrip(self):
        self.server.route(
            "/locations",
            lambda q: (
                200,
                [
                    {"type": "stop", "id": "8011160", "name": "Berlin Hbf",
                     "location": {"type": "location", "latitude": 52.525, "longitude": 13.369}},
                    {"type": "location", "name": "Alexanderplatz", "poi": True},
                ],
                {},
            ),
        )
        result = self.client.locations().query("Berlin").results(5).get()
        self.assertEqual(len(result), 2)
        self.assertEqual(result[0].id, "8011160")
        self.assertAlmostEqual(result[0].location.latitude, 52.525)
        from transport_rest.models_gen import Location

        self.assertIsInstance(result[1], Location)
        self.assertTrue(result[1].poi)

    def test_locations_requires_query(self):
        with self.assertRaises(e.InvalidParameterError):
            self.client.locations().get()


class TestDepartures(TransportRestTestCase):
    def test_departures_parses_and_sends_products(self):
        captured = {}

        def handler(query):
            captured.update(query)
            return 200, {
                "departures": [{
                    "tripId": "t1",
                    "line": {"id": "ICE 599", "mode": "train"},
                    "when": "2026-08-01T12:00:00+02:00",
                    "plannedWhen": "2026-08-01T11:58:00+02:00",
                    "delay": 120.0,
                    "platform": "8",
                }],
                "realtimeDataUpdatedAt": 1754000000,
            }, {}

        self.server.route("/stops/8011160/departures", handler)
        result = (
            self.client.departures("8011160")
            .results(10)
            .products(lambda p: p.bus(False).tram(False))
            .more_stops(["8010159"])
            .when(WHEN)
            .get()
        )
        self.assertEqual(captured.get("bus"), "false")
        self.assertEqual(captured.get("tram"), "false")
        self.assertEqual(captured.get("moreStops"), "8010159")
        self.assertEqual(captured.get("when"), "2026-08-01T12:00:00+02:00")
        self.assertEqual(result.departures[0].delay, 120)
        self.assertEqual(result.realtime_data_updated_at, 1754000000)
        self.assertEqual(result.departures[0].line.mode, "train")


class TestJourneys(TransportRestTestCase):
    def test_journeys_happy_path(self):
        captured = {}

        def handler(query):
            captured.update(query)
            return 200, {
                "journeys": [{
                    "refreshToken": "ref/token|1",
                    "legs": [{
                        "tripId": "t9",
                        "origin": {"type": "stop", "id": "8011160"},
                        "destination": {"type": "stop", "id": "8000108"},
                    }],
                }],
                "earlierRef": "E", "laterRef": "L",
            }, {}

        self.server.route("/journeys", handler)
        result = (
            self.client.journeys(JourneyPlace.stop_id("8011160"),
                                 JourneyPlace.name("Leipzig Hbf"))
            .via(JourneyPlace.poi("poi1", 51.5, 12.2))
            .transfers(0)
            .earlier_than("REF1")
            .get()
        )
        self.assertEqual(captured.get("from"), "8011160")
        self.assertEqual(captured.get("to.name"), "Leipzig Hbf")
        self.assertEqual(captured.get("earlierThan"), "REF1")
        leg = result.journeys[0].legs[0]
        self.assertEqual(leg.trip_id, "t9")

    def test_conflicting_times_rejected_before_request(self):
        with self.assertRaises(e.InvalidParameterError):
            self.client.journeys(
                JourneyPlace.stop_id("a"), JourneyPlace.stop_id("b")
            ).departure(WHEN).arrival(WHEN).get()
        self.assertEqual(self.server.requests, [])


class TestErrors(TransportRestTestCase):
    def test_api_error_404(self):
        self.server.route("/stops/nope", lambda q: (404, {"message": "Stop not found."}, {}))
        try:
            self.client.stop("nope").get()
            self.fail("expected ApiError")
        except e.ApiError as err:
            self.assertEqual(err.status, 404)
            self.assertEqual(err.message, "Stop not found.")

    def test_rate_limited_with_retry_after(self):
        self.server.route(
            "/locations",
            lambda q: (429, {"message": "Too Many Requests"}, {"Retry-After": "30"}),
        )
        try:
            self.client.locations().query("x").get()
            self.fail("expected RateLimitedError")
        except e.RateLimitedError as err:
            self.assertEqual(err.retry_after.total_seconds(), 30)

    def test_non_json_502_is_http_error(self):
        self.server.route("/locations", lambda q: (502, "<html>Bad Gateway</html>", {}))
        try:
            self.client.locations().query("x").get()
            self.fail("expected HttpError")
        except e.HttpError as err:
            self.assertEqual(err.status, 502)
            self.assertIn("<html>", err.body_snippet)

    def test_malformed_json_is_serialization_error(self):
        self.server.route("/stops/x/departures", lambda q: (200, "{not json", {}))
        try:
            self.client.departures("x").get()
            self.fail("expected SerializationError")
        except e.SerializationError:
            pass


class TestCapabilities(TransportRestTestCase):
    def test_radar_gated_on_db(self):
        with self.assertRaises(e.CapabilityNotSupportedError):
            self.client.radar().north(52.53).west(13.36).south(52.51).east(13.39).get()

    def test_radar_allowed_on_bvg(self):
        bvg = TransportRestClient(provider="bvg", base_url=self.server.url)
        self.server.route(
            "/radar", lambda q: (200, {"movements": [{"tripId": "m1"}]}, {})
        )
        result = bvg.radar().north(52.53).west(13.36).south(52.51).east(13.39).get()
        self.assertEqual(result.movements[0].trip_id, "m1")


class TestUnknownFields(unittest.TestCase):
    def test_future_fields_are_tolerated(self):
        from transport_rest.models_gen import Stop

        stop = Stop.from_dict({"type": "stop", "id": "futuristic", "brandNewField": [1, 2]})
        self.assertEqual(stop.id, "futuristic")


class TestAsync(unittest.TestCase):
    def test_get_async_returns_same_data(self):
        async def scenario():
            with MockServer() as server:
                server.route(
                    "/locations",
                    lambda q: (200, [{"type": "stop", "id": "s1"}], {}),
                )
                client = TransportRestClient(base_url=server.url)
                return await client.locations().query("Berlin").get_async()

        result = asyncio.run(scenario())
        self.assertEqual(result[0].id, "s1")


if __name__ == "__main__":
    unittest.main()
