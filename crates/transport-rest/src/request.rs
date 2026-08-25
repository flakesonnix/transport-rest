//! Request planning: URL building and query parameter serialization.

use crate::error::{InvalidParameterError, TransportRestError};

/// Ordered query parameter list preserving insertion order.
#[derive(Debug, Default, Clone)]
pub(crate) struct Query(Vec<(String, String)>);

impl Query {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    /// Add a parameter; skips `None`.
    pub(crate) fn opt(&mut self, key: &str, value: Option<impl std::fmt::Display>) {
        if let Some(v) = value {
            self.push(key, v.to_string());
        }
    }

    /// Add a raw parameter.
    pub(crate) fn push(&mut self, key: &str, value: String) {
        self.0.push((key.to_owned(), value));
    }

    /// True if no parameters were added.
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Encode into a URL query string (`key=value&...`) with percent encoding.
    pub(crate) fn encode(self) -> String {
        let mut out = String::new();
        for (i, (k, v)) in self.0.into_iter().enumerate() {
            if i > 0 {
                out.push('&');
            }
            out.push_str(&encode_component(k));
            out.push('=');
            out.push_str(&encode_component(v));
        }
        out
    }
}

/// Percent-encode one query component (RFC 3986, spaces as `%20`).
fn encode_component(s: impl AsRef<str>) -> String {
    const SET: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC
        .remove(b'*')
        .remove(b'-')
        .remove(b'.')
        .remove(b'_');
    percent_encoding::utf8_percent_encode(s.as_ref(), SET).to_string()
}

/// A place referenced by journey queries (`from`/`to`/`via`).
///
/// Mirrors the four accepted wire forms:
/// * stop ID (`from=<id>`)
/// * stop/station name (`from.name=<name>`)
/// * POI (`from.id`, `from.latitude`, `from.longitude`)
/// * address (`from.latitude`, `from.longitude`, `from.address`)
#[derive(Debug, Clone, PartialEq)]
pub enum JourneyPlace {
    /// A stop/station ID (e.g. `"8011160"` for Berlin Hbf).
    StopId(String),
    /// A free-text stop/station name (e.g. `"Berlin Hbf"`).
    Name(String),
    /// A POI identified by ID plus coordinates.
    Poi {
        /// POI ID.
        id: String,
        /// Latitude.
        latitude: f64,
        /// Longitude.
        longitude: f64,
    },
    /// An address near the given coordinates.
    Address {
        /// Latitude.
        latitude: f64,
        /// Longitude.
        longitude: f64,
        /// Street address string.
        address: String,
    },
}

impl From<&str> for JourneyPlace {
    fn from(id: &str) -> Self {
        Self::StopId(id.to_owned())
    }
}

impl From<String> for JourneyPlace {
    fn from(id: String) -> Self {
        Self::StopId(id)
    }
}

impl JourneyPlace {
    /// Serialize into query parameters under the given prefix
    /// (`"from"`, `"to"` or `"via"`).
    pub(crate) fn encode(&self, prefix: &str, q: &mut Query) {
        match self {
            Self::StopId(id) => q.push(prefix, id.clone()),
            Self::Name(name) => q.push(&format!("{prefix}.name"), name.clone()),
            Self::Poi {
                id,
                latitude,
                longitude,
            } => {
                q.push(&format!("{prefix}.id"), id.clone());
                q.push(&format!("{prefix}.latitude"), latitude.to_string());
                q.push(&format!("{prefix}.longitude"), longitude.to_string());
            }
            Self::Address {
                latitude,
                longitude,
                address,
            } => {
                q.push(&format!("{prefix}.latitude"), latitude.to_string());
                q.push(&format!("{prefix}.longitude"), longitude.to_string());
                q.push(&format!("{prefix}.address"), address.clone());
            }
        }
    }

    pub(crate) fn validate(&self) -> Result<(), TransportRestError> {
        match self {
            Self::Poi { id, .. } if id.trim().is_empty() => Err(invalid_poi()),
            Self::Address { address, .. } if address.trim().is_empty() => {
                Err(TransportRestError::InvalidParameter(
                    InvalidParameterError::new("address", "address must not be empty"),
                ))
            }
            _ => Ok(()),
        }
    }
}

fn invalid_poi() -> TransportRestError {
    TransportRestError::InvalidParameter(InvalidParameterError::new(
        "poi",
        "POI id must not be empty",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode_of(f: impl FnOnce(&mut Query)) -> String {
        let mut q = Query::new();
        f(&mut q);
        q.encode()
    }

    #[test]
    fn journey_place_encoding_forms() {
        let s = encode_of(|q| {
            JourneyPlace::StopId("8011160".into()).encode("from", q);
            JourneyPlace::Name("Leipzig Hbf".into()).encode("to", q);
        });
        assert!(s.contains("from=8011160"));
        assert!(s.contains("to.name=Leipzig%20Hbf"), "got {s}");
    }

    #[test]
    fn poi_and_address_forms() {
        let s = encode_of(|q| {
            JourneyPlace::Poi {
                id: "p1".into(),
                latitude: 51.5,
                longitude: 12.2,
            }
            .encode("from", q);
        });
        assert_eq!(s, "from.id=p1&from.latitude=51.5&from.longitude=12.2");

        let s = encode_of(|q| {
            JourneyPlace::Address {
                latitude: 52.5,
                longitude: 13.4,
                address: "Alexanderplatz 1".into(),
            }
            .encode("via", q);
        });
        assert_eq!(
            s,
            "via.latitude=52.5&via.longitude=13.4&via.address=Alexanderplatz%201"
        );
    }

    #[test]
    fn special_characters_are_escaped() {
        let s = encode_of(|q| q.push("query", "a&b=c d/e?".into()));
        assert_eq!(s, "query=a%26b%3Dc%20d%2Fe%3F");
    }

    #[test]
    fn path_segments_are_percent_encoded() {
        let seg = crate::util::encode_path_segment("2|#A|1#1005##");
        assert!(!seg.contains('#'));
        assert!(!seg.contains('/'));
    }
}
