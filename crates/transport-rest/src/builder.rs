//! Builder for [`TransportRestClient`].

use std::sync::Arc;
use std::time::Duration;

use url::Url;

use crate::{Capability, ClientState, Provider, TransportRestClient};
use crate::error::{InvalidParameterError, TransportRestError};

const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Guard against oversized/hostile responses.
const DEFAULT_MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;
pub(crate) const DEFAULT_USER_AGENT: &str =
    concat!("transport-rest-rs/", env!("CARGO_PKG_VERSION"));

/// Builder for a [`TransportRestClient`].
///
/// ```
/// # use std::time::Duration;
/// # use transport_rest::{Provider, TransportRestClient};
/// let client = TransportRestClient::builder()
///     .provider(Provider::Bvg)
///     .timeout(Duration::from_secs(15))
///     .user_agent("my-app/1.0 (+https://example.org)")
///     .build()
///     .unwrap();
/// assert_eq!(client.base_url().as_str(), "https://v6.bvg.transport.rest/");
/// ```
#[derive(Debug, Clone)]
pub struct TransportRestClientBuilder {
    provider: Provider,
    base_url: Option<String>,
    timeout: Duration,
    connect_timeout: Duration,
    user_agent: String,
    max_response_bytes: usize,
    enabled_capabilities: Vec<Capability>,
    proxy: Option<reqwest::Proxy>,
}

impl Default for TransportRestClientBuilder {
    fn default() -> Self {
        Self {
            provider: Provider::Db,
            base_url: None,
            timeout: DEFAULT_REQUEST_TIMEOUT,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            user_agent: DEFAULT_USER_AGENT.to_owned(),
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
            enabled_capabilities: Vec::new(),
            proxy: None,
        }
    }
}

impl TransportRestClientBuilder {
    /// Select a provider instance. Defaults to [`Provider::Db`].
    pub fn provider(mut self, provider: Provider) -> Self {
        self.provider = provider;
        self
    }

    /// Override the base URL (e.g. for self-hosted instances or tests).
    /// Takes precedence over the provider default.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Overall request timeout. Defaults to 30s.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Connection establishment timeout. Defaults to 10s.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// `User-Agent` header value. A descriptive UA is appreciated by the
    /// public instance operators; defaults to `transport-rest-rs/<version>`.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Maximum accepted response body size in bytes. Defaults to 16 MiB.
    pub fn max_response_bytes(mut self, max: usize) -> Self {
        self.max_response_bytes = max.max(1);
        self
    }

    /// Route requests through an HTTP(S)/SOCKS proxy.
    pub fn proxy(mut self, proxy: reqwest::Proxy) -> Self {
        self.proxy = Some(proxy);
        self
    }

    /// Explicitly enable a capability even though it is not known to be
    /// supported by the chosen provider. Useful for custom instances that
    /// do support `/radar` & co.
    pub fn enable_capability(mut self, capability: Capability) -> Self {
        if !self.enabled_capabilities.contains(&capability) {
            self.enabled_capabilities.push(capability);
        }
        self
    }

    /// Validate configuration and construct the client.
    pub fn build(self) -> Result<TransportRestClient, TransportRestError> {
        let base_url = match (&self.base_url, &self.provider) {
            (Some(url), _) => url.clone(),
            (None, Provider::Custom { base_url }) => base_url.clone(),
            (None, provider) => provider
                .default_base_url()
                .map(str::to_owned)
                .ok_or_else(|| {
                    TransportRestError::InvalidParameter(InvalidParameterError::other(
                        "custom provider requires base_url",
                    ))
                })?,
        };
        let mut base_url = Url::parse(&base_url).map_err(|_| {
            TransportRestError::InvalidParameter(InvalidParameterError::new(
                "base_url",
                format!("'{base_url}' is not a valid absolute http(s) URL"),
            ))
        })?;
        if base_url.scheme() != "https" && base_url.scheme() != "http" {
            return Err(TransportRestError::InvalidParameter(
                InvalidParameterError::new("base_url", "scheme must be http or https"),
            ));
        }
        // Normalize: requests always append paths like /locations, so strip
        // any trailing slash and query/fragment.
        base_url.set_path("");
        base_url.set_query(None);
        base_url.set_fragment(None);

        let mut capabilities = self.provider.default_capabilities();
        for cap in &self.enabled_capabilities {
            capabilities.insert_raw(*cap);
        }

        let mut http = reqwest::Client::builder()
            .user_agent(self.user_agent)
            .timeout(self.timeout)
            .connect_timeout(self.connect_timeout)
            .redirect(reqwest::redirect::Policy::limited(5))
            .use_rustls_tls();
        if let Some(proxy) = self.proxy {
            http = http.proxy(proxy);
        }
        let http = http.build().map_err(|e| {
            TransportRestError::Network(crate::error::NetworkError {
                url: Some(base_url.clone()),
                source: e,
            })
        })?;

        Ok(TransportRestClient {
            state: Arc::new(ClientState {
                http,
                base_url,
                provider: self.provider,
                capabilities,
                max_response_bytes: self.max_response_bytes,
            }),
        })
    }
}
