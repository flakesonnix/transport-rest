"""transport.rest client core: HTTP plumbing shared by the endpoint builders."""

from __future__ import annotations

import datetime as _dt
import json as _json
import urllib.error
import urllib.parse
import urllib.request
from typing import Any, Callable, Optional

from . import errors as _e

PROVIDERS = {
    "db": "https://v6.db.transport.rest",
    "bvg": "https://v6.bvg.transport.rest",
    "vbb": "https://v6.vbb.transport.rest",
    "poland": "https://poland.transport.rest",
}

_DEFAULT_UA = "transport-rest-py/0.1.0"
_DEFAULT_MAX_BYTES = 16 * 1024 * 1024

_PROVIDER_CAPABILITIES = {
    "db": {"stations"},
    "bvg": {"stops_search", "radar", "reachable_from", "trips_by_name"},
    "vbb": {"stops_search", "radar", "reachable_from", "trips_by_name"},
    "poland": {"radar", "reachable_from", "trips_by_name"},
}


def _encode_path_segment(value: str) -> str:
    return urllib.parse.quote(value, safe="")


def _parse_retry_after(value: str) -> Optional[_dt.timedelta]:
    value = value.strip()
    if value.isdigit():
        return _dt.timedelta(seconds=int(value))
    try:
        from email.utils import parsedate_to_datetime

        when = parsedate_to_datetime(value)
        delta = when - _dt.datetime.now(_dt.timezone.utc)
        return delta if delta.total_seconds() > 0 else None
    except (TypeError, ValueError):
        return None


class TransportRestClient:
    """Client for one transport.rest instance.

    Args:
        provider: one of ``db``/``bvg``/``vbb``/``poland``.
        base_url: overrides the provider default (self-hosted instances).
        timeout: overall request timeout in seconds.
        user_agent: descriptive User-Agent; defaults to ``transport-rest-py/x``.
        max_response_bytes: guard against oversized responses.
        enable_capabilities: force-enable endpoint groups for custom instances.
    """

    def __init__(
        self,
        provider: str = "db",
        *,
        base_url: Optional[str] = None,
        timeout: float = 30.0,
        user_agent: str = _DEFAULT_UA,
        max_response_bytes: int = _DEFAULT_MAX_BYTES,
        enable_capabilities: Optional[list] = None,
    ) -> None:
        if base_url is None and provider not in PROVIDERS:
            raise _e.InvalidParameterError("base_url", "custom providers require a base_url")
        url = base_url or PROVIDERS[provider]
        parsed = urllib.parse.urlparse(url)
        if parsed.scheme not in ("http", "https") or not parsed.netloc:
            raise _e.InvalidParameterError("base_url", f"'{url}' is not a valid http(s) URL")
        self._base_url = url.rstrip("/")
        self._provider = provider
        self._timeout = timeout
        self._user_agent = user_agent
        self._max_response_bytes = max_response_bytes
        caps = set(_PROVIDER_CAPABILITIES.get(provider, set()))
        caps.update(enable_capabilities or [])
        self._capabilities = caps

    # -- introspection -------------------------------------------------------

    @property
    def provider(self) -> str:
        return self._provider

    @property
    def base_url(self) -> str:
        return self._base_url

    def supports(self, capability: str) -> bool:
        return capability in self._capabilities

    # -- resource accessors --------------------------------------------------

    def locations(self) -> "LocationsBuilder":
        from .builders import LocationsBuilder

        return LocationsBuilder(self)

    def nearby(self) -> "NearbyBuilder":
        from .builders import NearbyBuilder

        return NearbyBuilder(self)

    def stop(self, stop_id: str) -> "StopBuilder":
        from .builders import StopBuilder

        return StopBuilder(self, stop_id)

    def departures(self, stop_id: str) -> "DeparturesBuilder":
        from .builders import DeparturesBuilder

        return DeparturesBuilder.create(self, stop_id)

    def arrivals(self, stop_id: str) -> "ArrivalsBuilder":
        from .builders import ArrivalsBuilder

        return ArrivalsBuilder.create(self, stop_id)

    def journeys(self, frm, to) -> "JourneysBuilder":
        from .builders import JourneysBuilder

        return JourneysBuilder(self, frm, to)

    def refresh_journey(self, refresh_token: str) -> "RefreshJourneyBuilder":
        from .builders import RefreshJourneyBuilder

        return RefreshJourneyBuilder(self, refresh_token)

    def trip(self, trip_id: str) -> "TripBuilder":
        from .builders import TripBuilder

        return TripBuilder(self, trip_id)

    def radar(self) -> "RadarBuilder":
        from .builders import RadarBuilder

        return RadarBuilder(self)

    def reachable_from(self) -> "ReachableFromBuilder":
        from .builders import ReachableFromBuilder

        return ReachableFromBuilder(self)

    def trips_by_name(self, query: str = "*") -> "TripsByNameBuilder":
        from .builders import TripsByNameBuilder

        return TripsByNameBuilder(self, query)

    def stations(self) -> "StationsBuilder":
        from .builders import StationsBuilder

        return StationsBuilder(self)

    def station(self, station_id: str) -> "StationBuilder":
        from .builders import StationBuilder

        return StationBuilder(self, station_id)

    def stops_search(self) -> "StopsSearchBuilder":
        from .builders import StopsSearchBuilder

        return StopsSearchBuilder(self)

    # -- execution -----------------------------------------------------------

    def check_capability(self, capability: str) -> None:
        if capability not in self._capabilities:
            raise _e.CapabilityNotSupportedError(capability, self._provider)

    def get_json(
        self,
        path: str,
        params: list,
        model: Callable[[Any], Any],
        capability: Optional[str] = None,
    ) -> Any:
        if capability is not None:
            self.check_capability(capability)
        url = self._url_for(path, params)
        request = urllib.request.Request(url, headers={"Accept": "application/json", "User-Agent": self._user_agent})
        try:
            with urllib.request.urlopen(request, timeout=self._timeout) as response:
                status = getattr(response, "status", 200)
                body = self._read_capped(response, url)
        except urllib.error.HTTPError as err:
            raise self._error_from_http_error(err) from None
        except urllib.error.URLError as err:
            reason = err.reason
            if isinstance(reason, (_dt.timedelta,)):
                pass
            if isinstance(reason, TimeoutError) or "timed out" in str(reason).lower():
                raise _e.RequestTimeoutError("request", url) from None
            raise _e.NetworkError(reason, url) from None
        except TimeoutError:
            raise _e.RequestTimeoutError("request", url) from None

        if not (200 <= status < 300):
            raise self._error_from_status(status, body, url)

        try:
            data = _json.loads(body.decode("utf-8"))
        except (ValueError, UnicodeDecodeError) as err:
            raise _e.SerializationError(f"body is not valid JSON: {err}", url) from None
        try:
            return model(data)
        except Exception as err:
            raise _e.SerializationError(f"response did not match expected schema: {err}", url) from None

    async def get_json_async(self, path: str, params: list, model, capability=None) -> Any:
        import asyncio

        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(None, lambda: self.get_json(path, params, model, capability))

    # -- helpers -------------------------------------------------------------

    def _url_for(self, path: str, params: list) -> str:
        qs = urllib.parse.urlencode(params, doseq=True)
        url = f"{self._base_url}{path}"
        return f"{url}?{qs}" if qs else url

    def _read_capped(self, response: Any, url: str) -> bytes:
        length = response.headers.get("Content-Length")
        if length and int(length) > self._max_response_bytes:
            raise _e.SerializationError(
                f"response of {length} bytes exceeds configured maximum of {self._max_response_bytes}",
                url,
            )
        chunks = []
        total = 0
        while True:
            chunk = response.read(64 * 1024)
            if not chunk:
                break
            total += len(chunk)
            if total > self._max_response_bytes:
                raise _e.SerializationError(
                    f"response exceeds configured maximum of {self._max_response_bytes}", url
                )
            chunks.append(chunk)
        return b"".join(chunks)

    def _error_from_http_error(self, err: urllib.error.HTTPError) -> _e.TransportRestError:
        url = err.url or ""
        try:
            body = self._read_capped(err, url)
        except Exception:
            body = b""
        retry_after = err.headers.get("Retry-After") if err.headers else None
        return self._classify_error(err.code, body, url, retry_after)

    def _error_from_status(self, status: int, body: bytes, url: str) -> _e.TransportRestError:
        return self._classify_error(status, body, url, None)

    def _classify_error(self, status, body: bytes, url: str, retry_after_raw) -> _e.TransportRestError:
        text = body.decode("utf-8", "replace").strip()
        snippet = text[:512]
        try:
            parsed = _json.loads(text) if text else None
        except ValueError:
            parsed = None
        message = parsed.get("message") if isinstance(parsed, dict) else None
        if status == 429:
            retry_after = _parse_retry_after(retry_after_raw) if retry_after_raw else None
            return _e.RateLimitedError(url, message or "rate limited", parsed, retry_after)
        if isinstance(parsed, dict) and ("message" in parsed):
            return _e.ApiError(status, url, message or "unspecified API error", parsed)
        return _e.HttpError(status, "GET", url, snippet or "<no body>")
