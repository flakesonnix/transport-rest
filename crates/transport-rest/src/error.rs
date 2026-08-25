//! Structured error types for the transport.rest client.
//!
//! The taxonomy distinguishes network problems
//! ([`TransportRestError::Network`]), timeouts ([`TransportRestError::Timeout`]),
//! uninterpretable HTTP responses ([`TransportRestError::Http`]), structured API
//! errors ([`TransportRestError::Api`]), rate limiting
//! ([`TransportRestError::RateLimited`]) and client-side problems such as invalid
//! parameters ([`TransportRestError::InvalidParameter`]).
//!
//! No library code panics on malformed input; unexpected response shapes surface
//! as [`TransportRestError::Serialization`].

use std::time::Duration;

use serde_json::Value;
use url::Url;

/// Convenience alias used throughout the crate.
pub type Result<T, E = TransportRestError> = std::result::Result<T, E>;

/// Maximum number of bytes kept from an error body for debugging purposes.
pub(crate) const ERROR_BODY_SNIPPET_LEN: usize = 512;

/// Top level error type of this library.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TransportRestError {
    /// A connection-level failure (DNS, TCP, TLS).
    #[error(transparent)]
    Network(#[from] NetworkError),

    /// The request exceeded the configured timeout.
    #[error(transparent)]
    Timeout(#[from] TimeoutError),

    /// The server answered with a status code we could not interpret,
    /// or with a body that violated basic HTTP expectations.
    #[error(transparent)]
    Http(#[from] HttpError),

    /// The transport.rest instance returned a structured error.
    #[error(transparent)]
    Api(#[from] ApiError),

    /// The instance rate limited us (HTTP 429).
    #[error(transparent)]
    RateLimited(#[from] RateLimitedError),

    /// The response body could not be parsed.
    #[error(transparent)]
    Serialization(#[from] SerializationError),

    /// A parameter was missing or invalid before any request was sent.
    #[error(transparent)]
    InvalidParameter(InvalidParameterError),

    /// The configured provider does not support the requested endpoint
    /// (e.g. `/radar` is not available on `v6.db.transport.rest`).
    #[error(transparent)]
    CapabilityNotSupported(CapabilityNotSupportedError),
}

/// Where in the request lifecycle a timeout occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutKind {
    /// Establishing the TCP/TLS connection timed out.
    Connect,
    /// The overall request (connect + send + receive) timed out.
    Request,
}

impl std::fmt::Display for TimeoutKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeoutKind::Connect => f.write_str("connect"),
            TimeoutKind::Request => f.write_str("request"),
        }
    }
}

/// Connection-level failure details.
#[derive(Debug, thiserror::Error)]
pub struct NetworkError {
    /// URL the failing request targeted, when already known.
    pub url: Option<Url>,
    /// Underlying reqwest error.
    #[source]
    pub source: reqwest::Error,
}

/// Timeout failure details.
#[derive(Debug, thiserror::Error)]
pub struct TimeoutError {
    /// Which phase of the request timed out.
    pub kind: TimeoutKind,
    /// URL the timing out request targeted, when already known.
    pub url: Option<Url>,
    /// Underlying reqwest error.
    #[source]
    pub source: reqwest::Error,
}

/// A non-success HTTP response that was *not* a structured API error
/// (e.g. an HTML error page from a proxy, or an empty body).
#[derive(Debug, thiserror::Error)]
pub struct HttpError {
    /// HTTP status code.
    pub status: u16,
    /// HTTP method of the request.
    pub method: String,
    /// The full request URL.
    pub url: Url,
    /// Truncated body to aid debugging without leaking unbounded payloads.
    pub body_snippet: String,
}

/// Structured error returned by transport.rest instances
/// (`{"message": "..."}` with a 4xx/5xx status).
#[derive(Debug, thiserror::Error)]
pub struct ApiError {
    /// HTTP status code of the response.
    pub status: u16,
    /// The full request URL.
    pub url: Url,
    /// Human readable message extracted from the response body.
    pub message: String,
    /// Raw body as returned by the instance, useful for debugging.
    pub body: Value,
}

/// HTTP 429 details, including an optional `Retry-After` hint.
#[derive(Debug, thiserror::Error)]
pub struct RateLimitedError {
    /// The underlying API error carrying URL/message/body details.
    #[source]
    pub api: ApiError,
    /// Suggested wait time parsed from the `Retry-After` header
    /// (seconds or HTTP date).
    pub retry_after: Option<Duration>,
}

/// Response body could not be deserialized into the expected model.
#[derive(Debug, thiserror::Error)]
pub struct SerializationError {
    /// URL the offending response came from.
    pub url: Option<Url>,
    /// Underlying cause.
    #[source]
    pub source: SerializationErrorKind,
}

/// Underlying cause of a [`SerializationError`].
#[derive(Debug, thiserror::Error)]
pub enum SerializationErrorKind {
    /// The body was not valid JSON at all.
    #[error("body is not valid JSON")]
    InvalidJson(#[source] serde_json::Error),
    /// Valid JSON but it did not match the expected model.
    ///
    /// Unknown *fields* never cause this; only structural mismatches do.
    #[error("response did not match expected schema: {0}")]
    Schema(#[source] serde_json::Error),
}

impl SerializationError {
    pub(crate) fn invalid_json(err: serde_json::Error, url: Option<Url>) -> Self {
        Self {
            url,
            source: SerializationErrorKind::InvalidJson(err),
        }
    }

    pub(crate) fn schema(err: serde_json::Error, url: Option<Url>) -> Self {
        Self {
            url,
            source: SerializationErrorKind::Schema(err),
        }
    }
}

/// A parameter failed client-side validation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid parameter '{}': {}", parameter.unwrap_or("<none>"), reason)]
pub struct InvalidParameterError {
    /// Name of the offending builder parameter, if applicable.
    pub parameter: Option<&'static str>,
    /// Why the value was rejected.
    pub reason: String,
}

impl InvalidParameterError {
    pub(crate) fn new(parameter: &'static str, reason: impl Into<String>) -> Self {
        Self {
            parameter: Some(parameter),
            reason: reason.into(),
        }
    }

    pub(crate) fn other(reason: impl Into<String>) -> Self {
        Self {
            parameter: None,
            reason: reason.into(),
        }
    }
}

/// An endpoint that is not available on every provider was requested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityNotSupportedError {
    /// Capability that was requested.
    pub capability: crate::Capability,
    /// Provider it was requested from.
    pub provider: crate::Provider,
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.url {
            Some(url) => write!(f, "network error for {}: {}", url, self.source),
            None => write!(f, "network error: {}", self.source),
        }
    }
}

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.url {
            Some(url) => write!(f, "request timed out ({}) for {}: {}", self.kind, url, self.source),
            None => write!(f, "request timed out ({}): {}", self.kind, self.source),
        }
    }
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "unexpected HTTP response: HTTP {} from {} {}: {}",
            self.status, self.method, self.url, self.body_snippet
        )
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::fmt::Display for RateLimitedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.retry_after {
            Some(d) => write!(f, "rate limited (HTTP 429), retry after {d:?}: {}", self.api.message),
            None => write!(f, "rate limited (HTTP 429): {}", self.api.message),
        }
    }
}

impl std::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.url {
            Some(url) => write!(f, "failed to deserialize response for {}: {}", url, self.source),
            None => write!(f, "failed to deserialize response: {}", self.source),
        }
    }
}


/// Internal helper: classify a [`reqwest::Error`] into our taxonomy.
///
/// Body-decoding problems never reach this function: bodies are read as raw
/// bytes and parsed by [`serde_json`], so malformed JSON surfaces as
/// [`TransportRestError::Serialization`] instead.
pub(crate) fn map_reqwest_error(url: Option<Url>, err: reqwest::Error) -> TransportRestError {
    if err.is_timeout() {
        let kind = if err.is_connect() {
            TimeoutKind::Connect
        } else {
            TimeoutKind::Request
        };
        TransportRestError::Timeout(TimeoutError { kind, url, source: err })
    } else {
        TransportRestError::Network(NetworkError { url, source: err })
    }
}

impl std::fmt::Display for CapabilityNotSupportedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "capability '{}' is not supported by provider '{:?}'; enable it explicitly via \
             TransportRestClientBuilder::enable_capability if you know the endpoint exists",
            self.capability, self.provider
        )
    }
}

impl std::error::Error for CapabilityNotSupportedError {}
