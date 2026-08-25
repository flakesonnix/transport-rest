//! Unit tests: query encoding, enums, error classification, builder validation.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use transport_rest::{TransportRestClient, TransportRestError};

#[test]
fn open_enum_fallbacks_roundtrip() {
    use transport_rest::models::Mode;
    assert_eq!(Mode::Train.as_str(), "train");
    assert_eq!(Mode::from("hyperloop"), Mode::Other("hyperloop".into()));
    assert!(!Mode::from("train").is_other());

    let json = serde_json::json!("segway");
    let mode: Mode = serde_json::from_value(json).unwrap();
    assert_eq!(mode, Mode::Other("segway".into()));
    // serialization preserves the unknown value
    assert_eq!(
        serde_json::to_value(&mode).unwrap(),
        serde_json::json!("segway")
    );
}

#[test]
fn client_builder_rejects_invalid_base_url() {
    let err = TransportRestClient::builder()
        .base_url("not a url")
        .build()
        .unwrap_err();
    assert!(matches!(err, TransportRestError::InvalidParameter(_)));
}

#[test]
fn custom_provider_requires_base_url() {
    let err = TransportRestClient::builder()
        .provider(transport_rest::Provider::Custom {
            base_url: String::new(),
        })
        .build()
        .unwrap_err();
    assert!(matches!(err, TransportRestError::InvalidParameter(_)));
}

#[test]
fn path_segments_are_percent_encoded() {
    // trip ids and refresh tokens contain slashes; they must not break URLs
    let seg = transport_rest_test_helper::encode("2|#A|1#1005##");
    assert!(!seg.contains('#'));
    assert!(!seg.contains('/'));
}

mod transport_rest_test_helper {
    pub fn encode(s: &str) -> String {
        transport_rest::__test_encode_path_segment(s)
    }
}
