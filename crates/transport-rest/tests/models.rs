//! Model deserialization tests against wire-format fixtures taken from the
//! official transport.rest documentation (docs/API_ANALYSIS.md sources).

use transport_rest::models::*;

/// Example `GET /stops/8010159` response from v6.db.transport.rest docs.
const STOP_HALLE: &str = r#"{
    "type": "stop",
    "id": "8010159",
    "ids": {
        "dhid": "de:15002:8010159",
        "MDV": "8010159",
        "NASA": "8010159"
    },
    "name": "Halle (Saale) Hbf",
    "location": {
        "type": "location",
        "id": "8010159",
        "latitude": 51.477079,
        "longitude": 11.98699
    },
    "products": {
        "nationalExpress": true,
        "national": true,
        "regionalExpress": true,
        "regional": true,
        "suburban": true,
        "bus": true,
        "ferry": false,
        "subway": false,
        "tram": true,
        "taxi": false
    }
}"#;

#[test]
fn parse_documented_stop() {
    let parsed: LocationResult = serde_json::from_str(STOP_HALLE).unwrap();
    let stop = parsed.as_stop().expect("tagged as stop");
    assert_eq!(stop.id, "8010159");
    assert_eq!(stop.name.as_deref(), Some("Halle (Saale) Hbf"));
    assert_eq!(
        parsed.coordinates(),
        Some((51.477079, 11.98699))
    );
    let dhid = stop
        .ids
        .as_ref()
        .and_then(|ids| ids.get("dhid").map(String::as_str));
    assert_eq!(dhid, Some("de:15002:8010159"));
}

#[test]
fn roundtrip_stop_preserves_type_tag() {
    let parsed: LocationResult = serde_json::from_str(STOP_HALLE).unwrap();
    let json = serde_json::to_value(&parsed).unwrap();
    assert_eq!(json["type"], "stop");
    assert_eq!(json["location"]["type"], "location");
}

#[test]
fn unknown_location_type_is_captured() {
    let raw = r#"{"type": "hovercraft", "id": "x", "name": "Future Thing"}"#;
    let parsed: LocationResult = serde_json::from_str(raw).unwrap();
    assert!(parsed.id().is_none(), "unknown types expose no typed accessors");
    let other = parsed.as_other_value().expect("captured verbatim");
    assert_eq!(other["type"], "hovercraft");
}

#[test]
fn bare_location_without_type_is_accepted() {
    let parsed: LocationResult =
        serde_json::from_str(r#"{"latitude": 52.5, "longitude": 13.4}"#).unwrap();
    assert_eq!(parsed.coordinates(), Some((52.5, 13.4)));
}
