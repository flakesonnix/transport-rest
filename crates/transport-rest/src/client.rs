//! Request execution pipeline shared by all endpoint builders.

use serde::de::DeserializeOwned;

use crate::error::{
    map_reqwest_error, ApiError, HttpError, RateLimitedError, SerializationError,
    SerializationErrorKind, TransportRestError, ERROR_BODY_SNIPPET_LEN,
};
use crate::request::Query;
use crate::{Capability, TransportRestClient};

/// Hard limit for how long we wait on any single response body chunk beyond
/// the client timeout; reqwest's overall timeout governs this.
impl TransportRestClient {
    pub(crate) fn check_capability(&self, capability: Capability) -> Result<(), TransportRestError> {
        if self.state.capabilities.contains(capability) {
            Ok(())
        } else {
            Err(TransportRestError::CapabilityNotSupported(
                crate::error::CapabilityNotSupportedError {
                    capability,
                    provider: self.state.provider.clone(),
                },
            ))
        }
    }

    /// Build a full URL from a path that already has percent-encoded segments.
    pub(crate) fn url_for(&self, path: &str) -> url::Url {
        debug_assert!(path.starts_with('/'), "paths must be absolute");
        let mut url = self.state.base_url.clone();
        // Url::set_path percent-encodes invalid characters but preserves
        // already-encoded sequences; our segments are pre-encoded.
        url.set_path(path);
        url
    }

    /// Execute a GET request and deserialize the JSON response.
    pub(crate) async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        query: Query,
    ) -> Result<T, TransportRestError> {
        let mut url = self.url_for(path);
        if !query.is_empty() {
            url.set_query(Some(&query.encode()));
        }

        let response = self
            .state
            .http
            .get(url.clone())
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(|e| map_reqwest_error(Some(url.clone()), e))?;

        let status = response.status().as_u16();
        if (200..300).contains(&status) {
            self.read_success(response, &url).await
        } else {
            Err(Self::error_from_response(response, status, url).await)
        }
    }

    async fn read_success<T: DeserializeOwned>(
        &self,
        mut response: reqwest::Response,
        url: &url::Url,
    ) -> Result<T, TransportRestError> {
        let bytes = self.read_capped(&mut response, url).await?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| {
            TransportRestError::Serialization(SerializationError::invalid_json(
                e,
                Some(url.clone()),
            ))
        })?;
        serde_json::from_value(value).map_err(|e| {
            TransportRestError::Serialization(SerializationError::schema(
                e,
                Some(url.clone()),
            ))
        })
    }

    /// Read a response body while enforcing `max_response_bytes`.
    ///
    /// Guards against oversized or hostile responses; content-length hints are
    /// checked before reading, actual bytes are counted while streaming.
    async fn read_capped(
        &self,
        response: &mut reqwest::Response,
        url: &url::Url,
    ) -> Result<Vec<u8>, TransportRestError> {
        if let Some(len) = response.content_length() {
            if len as usize > self.state.max_response_bytes {
                return Err(TransportRestError::Serialization(SerializationError {
                    url: Some(url.clone()),
                    source: SerializationErrorKind::Schema(serde::de::Error::custom(format!(
                        "response of {len} bytes exceeds configured maximum of {} bytes",
                        self.state.max_response_bytes
                    ))),
                }));
            }
        }

        let mut buf = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| map_reqwest_error(Some(url.clone()), e))?
        {
            if buf.len() + chunk.len() > self.state.max_response_bytes {
                return Err(TransportRestError::Serialization(SerializationError {
                    url: Some(url.clone()),
                    source: SerializationErrorKind::Schema(serde::de::Error::custom(format!(
                        "response exceeds configured maximum of {} bytes",
                        self.state.max_response_bytes
                    ))),
                }));
            }
            buf.extend_from_slice(&chunk);
        }
        Ok(buf)
    }

    /// Map a non-2xx response onto [`TransportRestError`] variants.
    async fn error_from_response(
        response: reqwest::Response,
        status: u16,
        url: url::Url,
    ) -> TransportRestError {
        let headers = response.headers().clone();
        let body_bytes = response.bytes().await.unwrap_or_default();
        let body_text = String::from_utf8_lossy(&body_bytes);
        let parsed: Option<serde_json::Value> = serde_json::from_str(body_text.trim()).ok();

        let message = parsed
            .as_ref()
            .and_then(|v| v.get("message"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned();

        if status == 429 {
            let retry_after = headers
                .get(reqwest::header::RETRY_AFTER)
                .and_then(parse_retry_after);
            return TransportRestError::RateLimited(RateLimitedError {
                api: ApiError {
                    status,
                    url,
                    message: if message.is_empty() {
                        "rate limited".to_owned()
                    } else {
                        message
                    },
                    body: parsed.unwrap_or(serde_json::Value::Null),
                },
                retry_after,
            });
        }

        if !message.is_empty() || matches!(&parsed, Some(v) if v.is_object()) {
            return TransportRestError::Api(ApiError {
                status,
                url,
                message: if message.is_empty() {
                    "unspecified API error".to_owned()
                } else {
                    message
                },
                body: parsed.unwrap_or(serde_json::Value::Null),
            });
        }

        TransportRestError::Http(HttpError {
            status,
            method: "GET".to_owned(),
            url,
            body_snippet: truncate_snippet(body_text.trim()),
        })
    }
}

fn truncate_snippet(text: &str) -> String {
    if text.len() <= ERROR_BODY_SNIPPET_LEN {
        text.to_owned()
    } else {
        let mut cut = ERROR_BODY_SNIPPET_LEN;
        while !text.is_char_boundary(cut) {
            cut -= 1;
        }
        format!("{}…", &text[..cut])
    }
}

/// Parse a `Retry-After` header value: either delay-seconds or HTTP-date.
fn parse_retry_after(value: &reqwest::header::HeaderValue) -> Option<std::time::Duration> {
    let s = value.to_str().ok()?;
    if let Ok(secs) = s.parse::<u64>() {
        return Some(std::time::Duration::from_secs(secs));
    }
    let date = chrono::DateTime::parse_from_rfc2822(s).ok()?;
    let delta = date.signed_duration_since(chrono::Utc::now());
    (delta > chrono::TimeDelta::zero()).then(|| delta.to_std().ok()).flatten()
}
