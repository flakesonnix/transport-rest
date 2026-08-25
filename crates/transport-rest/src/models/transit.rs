//! Transit entities: operators, lines, remarks, prices.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::enums::{Mode, RemarkKind};
use super::place::LocationResult;
use super::Aliases;

/// A transit operator.
///
/// The upstream OpenAPI spec declares some `operator` fields as numbers while
/// the wire format uses objects (and occasionally bare ids); this model
/// accepts all three shapes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Operator {
    /// Operator ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Operator name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Operator {
    /// Normalize the tolerated wire shapes (object | string id) into an operator.
    fn from_lenient(v: Value) -> Self {
        match v {
            Value::Object(map) => Self {
                id: map.get("id").and_then(Value::as_str).map(str::to_owned),
                name: map.get("name").and_then(Value::as_str).map(str::to_owned),
            },
            Value::Number(n) => Self {
                id: Some(n.to_string()),
                name: None,
            },
            Value::String(s) => Self {
                id: Some(s),
                name: None,
            },
            other => Self {
                id: None,
                name: other.as_str().map(str::to_owned),
            },
        }
    }
}

/// Deserialize [`Operator`] tolerantly.
pub(crate) mod operator_lenient {
    use super::*;

    pub(crate) fn serialize<S: serde::Serializer>(op: &Option<Operator>, s: S) -> Result<S::Ok, S::Error> {
        match op {
            Some(op) => op.serialize(s),
            None => s.serialize_none(),
        }
    }

    pub(crate) fn deserialize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<Operator>, D::Error> {
        let v = Option::<Value>::deserialize(d)?;
        Ok(v.map(Operator::from_lenient))
    }
}

/// A public transport line.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Line {
    /// Line ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Displayed line name (e.g. `"ICE 599"`, `"M4"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Additional name context (e.g. route description).
    #[serde(rename = "additionalName", default, skip_serializing_if = "Option::is_none")]
    pub additional_name: Option<String>,
    /// Internal admin code of the line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_code: Option<String>,
    /// Trip number within the line (`Fahrtnummer`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fahrt_nr: Option<String>,
    /// Product key; profile-specific open string (`ice`, `bus`, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
    /// Human readable product name.
    #[serde(rename = "productName", default, skip_serializing_if = "Option::is_none")]
    pub product_name: Option<String>,
    /// Mode of transport on this line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<Mode>,
    /// Operating company.
    #[serde(
        default,
        skip_serializing_if = "operator_lenient_is_none",
        with = "super::transit::operator_lenient"
    )]
    pub operator: Option<Operator>,
    /// Express service?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub express: Option<bool>,
    /// Metro service?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metro: Option<bool>,
    /// Night service?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub night: Option<bool>,
    /// Numeric line number, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nr: Option<i64>,
    /// Symbol shown on maps/signs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Publicly visible?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
    /// Terminus names for both directions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub directions: Vec<String>,
    /// Route IDs served by this line.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub routes: Vec<String>,
}

fn operator_lenient_is_none(op: &Option<Operator>) -> bool {
    op.is_none()
}

/// Hint, status or warning attached to journeys, legs, departures, trips
/// or stops.
///
/// FPTF distinguishes `hint`/`status`/`warning` shapes; they overlap heavily,
/// so this library merges them into one struct with a [`Remark::kind`]
/// discriminator. Unknown kinds are preserved via [`RemarkKind::Other`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Remark {
    /// Discriminator of the remark.
    #[serde(rename = "type")]
    pub kind: RemarkKind,
    /// Machine readable code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Short summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Full text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Trip the remark applies to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trip_id: Option<String>,
    /// Warning ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Priority (higher = more severe).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<i64>,
    /// Free-form category label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Products affected by this warning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub products: Option<Aliases::Products>,
    /// Company that issued the warning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    /// Validity start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Validity end.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Last modification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified: Option<crate::datetime::DateTime<crate::datetime::FixedOffset>>,
    /// Lines affected by this warning.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_lines: Vec<super::transit::Line>,
    /// Stops where the disruption starts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub from_stops: Vec<LocationResult>,
    /// Stops where the disruption ends.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub to_stops: Vec<LocationResult>,
}

/// Price information for a journey.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Price {
    /// Amount in `currency`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    /// ISO 4217 currency code (e.g. `"EUR"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// Additional pricing hint (e.g. `"Sparpreis"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// Cycle times of a line/trip in minutes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub struct Cycle {
    /// Minimum headway.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<i64>,
    /// Maximum headway.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<i64>,
    /// Nominal number of trips per cycle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nr: Option<i64>,
}
