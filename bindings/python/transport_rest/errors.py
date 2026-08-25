"""Structured error taxonomy mirroring the Rust core."""

from __future__ import annotations

import datetime as _dt
from typing import Any, Optional

__all__ = [
    "TransportRestError",
    "NetworkError",
    "RequestTimeoutError",
    "TimeoutKind",
    "HttpError",
    "ApiError",
    "RateLimitedError",
    "SerializationError",
    "InvalidParameterError",
    "CapabilityNotSupportedError",
]


class TransportRestError(Exception):
    """Base class of all library errors."""


class TimeoutKind:
    CONNECT = "connect"
    REQUEST = "request"


class NetworkError(TransportRestError):
    """Connection-level failure (DNS, TCP, TLS)."""

    def __init__(self, source: BaseException, url: Optional[str] = None) -> None:
        self.source = source
        self.url = url
        super().__init__(f"network error{' for ' + url if url else ''}: {source}")


class RequestTimeoutError(TransportRestError):
    """The request exceeded the configured timeout."""

    def __init__(self, kind: str, url: Optional[str] = None) -> None:
        self.kind = kind
        self.url = url
        super().__init__(f"request timed out ({kind}){(' for ' + url) if url else ''}")


class HttpError(TransportRestError):
    """Non-success HTTP response that was not a structured API error."""

    def __init__(self, status: int, method: str, url: str, body_snippet: str = "") -> None:
        self.status = status
        self.method = method
        self.url = url
        self.body_snippet = body_snippet
        super().__init__(f"unexpected HTTP response: HTTP {status} from {method} {url}: {body_snippet}")


class ApiError(TransportRestError):
    """Structured error returned by transport.rest instances ({"message": ...})."""

    def __init__(self, status: int, url: str, message: str, body: Any = None) -> None:
        self.status = status
        self.url = url
        self.message = message
        self.body = body
        super().__init__(message or f"API error (HTTP {status})")


class RateLimitedError(ApiError):
    """HTTP 429 including an optional Retry-After hint."""

    def __init__(
        self,
        url: str,
        message: str = "rate limited",
        body: Any = None,
        retry_after: Optional[_dt.timedelta] = None,
    ) -> None:
        self.retry_after = retry_after
        super().__init__(429, url, message, body)

    def __str__(self) -> str:  # noqa: D105
        base = super().__str__()
        if self.retry_after is not None:
            return f"rate limited (HTTP 429), retry after {self.retry_after}: {base}"
        return f"rate limited (HTTP 429): {base}"


class SerializationError(TransportRestError):
    """The response body could not be deserialized into the expected model."""

    def __init__(self, reason: str, url: Optional[str] = None) -> None:
        self.reason = reason
        self.url = url
        super().__init__(f"failed to deserialize response{(' for ' + url) if url else ''}: {reason}")


class InvalidParameterError(TransportRestError, ValueError):
    """A parameter failed client-side validation."""

    def __init__(self, parameter: Optional[str], reason: str) -> None:
        self.parameter = parameter
        self.reason = reason
        name = parameter or "<none>"
        super().__init__(f"invalid parameter '{name}': {reason}")


class CapabilityNotSupportedError(TransportRestError):
    """Endpoint group unavailable on the configured provider."""

    def __init__(self, capability: str, provider: str) -> None:
        self.capability = capability
        self.provider = provider
        super().__init__(
            f"capability '{capability}' is not supported by provider '{provider}'; "
            "enable it explicitly via TransportRestClient(..., enable_capabilities=[...]) "
            "if you know the endpoint exists"
        )
