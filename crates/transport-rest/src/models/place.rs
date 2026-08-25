//! Places: [`Location`], [`Stop`], [`Station`] and their tagged unions.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::transit::Line;
use super::Aliases;

/// A point: POI, address or bare coordinate.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Location {
    /// Wire discriminator, normally `"location"`. Preserved for lossless
    /// round-trips; absent on bare coordinate objects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// Stable identifier, if the location is addressable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Human readable name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// True for points of interest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poi: Option<bool>,
    /// Street address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// Geographic latitude in degrees (WGS 84).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    /// Geographic longitude in degrees (WGS 84).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    /// Altitude in meters, rarely populated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub altitude: Option<f64>,
    /// Walking distance in meters (populated by nearby queries).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance: Option<f64>,
}

/// A single physical stop: a platform, a bus stop sign, ...
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Stop {
    /// Wire discriminator, normally `"stop"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// Provider-specific stop ID (e.g. EVA number at the DB instance).
    pub id: String,
    /// Human readable name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Parent station, if this stop belongs to one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub station: Option<Box<Station>>,
    /// Geographic information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    /// Which products serve this stop.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub products: Option<Aliases::Products>,
    /// External identifiers (e.g. `dhid`, `MDV`, `NASA`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ids: Option<Aliases::Ids>,
    /// Lines serving this stop (`linesOfStops=true`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<Line>,
    /// Entrances of the stop area.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entrances: Vec<Location>,
    /// Meta station grouping multiple stops?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_meta: Option<bool>,
    /// Travel center opening hours.
    #[serde(
        default,
        rename = "reisezentrumOpeningHours",
        skip_serializing_if = "Option::is_none"
    )]
    pub opening_hours: Option<Aliases::OpeningHours>,
    /// Expected occupancy; open value set (`low`, `medium`, `high`, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_factor: Option<String>,
    /// Transit authority operating here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transit_authority: Option<String>,
    /// Walking distance in meters (nearby results).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance: Option<f64>,
}

/// A larger station area that may contain multiple stops.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Station {
    /// Wire discriminator, normally `"station"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// Provider-specific station ID.
    pub id: String,
    /// Human readable name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Parent station (rarely nested).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub station: Option<Box<Station>>,
    /// Geographic information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    /// Which products serve this station.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub products: Option<Aliases::Products>,
    /// Lines serving this station (`linesOfStops=true`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<Line>,
    /// Meta station grouping multiple stops?
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_meta: Option<bool>,
    /// Region IDs the station belongs to.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub regions: Vec<String>,
    /// Free-form facility attributes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub facilities: Option<std::collections::BTreeMap<String, String>>,
    /// Travel center opening hours.
    #[serde(
        default,
        rename = "reisezentrumOpeningHours",
        skip_serializing_if = "Option::is_none"
    )]
    pub opening_hours: Option<Aliases::OpeningHours>,
    /// Sub-stops of this station.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stops: Vec<LocationResult>,
    /// Entrances of the station area.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entrances: Vec<Location>,
    /// Transit authority operating here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transit_authority: Option<String>,
    /// Walking distance in meters (nearby results).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance: Option<f64>,
}

/// Result of location search & nearby queries.
///
/// Discriminated by the wire `type` field; unknown future types are captured
/// in [`LocationResult::Other`] instead of failing.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum LocationResult {
    /// A stop.
    Stop(Stop),
    /// A station.
    Station(Station),
    /// A plain location (POI or address).
    Location(Location),
    /// An unrecognized object; kept verbatim for forward compatibility.
    Other(Value),
}

impl<'de> Deserialize<'de> for LocationResult {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(deserializer)?;
        Ok(match tag_of(&v) {
            Some("stop") => Self::Stop(from_value(v)?),
            Some("station") => Self::Station(from_value(v)?),
            // Bare locations sometimes omit `type`.
            Some("location") | None => Self::Location(from_value(v)?),
            _ => Self::Other(v),
        })
    }
}

/// A stop or a station.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum StopOrStation {
    /// A stop.
    Stop(Stop),
    /// A station.
    Station(Station),
    /// An unrecognized object; kept verbatim for forward compatibility.
    Other(Value),
}

impl<'de> Deserialize<'de> for StopOrStation {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(deserializer)?;
        Ok(match tag_of(&v) {
            Some("stop") => Self::Stop(from_value(v)?),
            Some("station") => Self::Station(from_value(v)?),
            _ => Self::Other(v),
        })
    }
}

/// Any place referenced by legs/trips/stopovers/frames.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Place {
    /// A stop.
    Stop(Stop),
    /// A station.
    Station(Station),
    /// A plain location.
    Location(Location),
    /// An unrecognized object; kept verbatim for forward compatibility.
    Other(Value),
}

impl<'de> Deserialize<'de> for Place {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(deserializer)?;
        Ok(match tag_of(&v) {
            Some("stop") => Self::Stop(from_value(v)?),
            Some("station") => Self::Station(from_value(v)?),
            Some("location") | None => Self::Location(from_value(v)?),
            _ => Self::Other(v),
        })
    }
}

fn tag_of(v: &Value) -> Option<&str> {
    v.get("type").and_then(Value::as_str)
}

fn from_value<T: serde::de::DeserializeOwned, E: serde::de::Error>(v: Value) -> Result<T, E> {
    serde_json::from_value(v).map_err(E::custom)
}

impl LocationResult {
    /// ID of the underlying object, if any.
    pub fn id(&self) -> Option<&str> {
        match self {
            Self::Stop(s) => Some(&s.id),
            Self::Station(s) => Some(&s.id),
            Self::Location(l) => l.id.as_deref(),
            Self::Other(_) => None,
        }
    }

    /// Name of the place, if any.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Stop(s) => s.name.as_deref(),
            Self::Station(s) => s.name.as_deref(),
            Self::Location(l) => l.name.as_deref(),
            Self::Other(_) => None,
        }
    }

    /// Coordinates as `(latitude, longitude)`, if known.
    pub fn coordinates(&self) -> Option<(f64, f64)> {
        match self {
            Self::Stop(s) => s.location.as_ref().and_then(coords_of),
            Self::Station(s) => s.location.as_ref().and_then(coords_of),
            Self::Location(l) => coords_of(l),
            Self::Other(_) => None,
        }
    }

    /// Borrow the inner [`Stop`], if this is one.
    pub fn as_stop(&self) -> Option<&Stop> {
        if let Self::Stop(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Borrow the inner [`Station`], if this is one.
    pub fn as_station(&self) -> Option<&Station> {
        if let Self::Station(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Borrow the inner [`Location`], if this is one.
    pub fn as_location(&self) -> Option<&Location> {
        if let Self::Location(l) = self {
            Some(l)
        } else {
            None
        }
    }

    /// Raw JSON for unknown values, else `None`.
    pub fn as_other_value(&self) -> Option<&Value> {
        if let Self::Other(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl StopOrStation {
    /// ID of the stop/station.
    pub fn id(&self) -> Option<&str> {
        match self {
            Self::Stop(s) => Some(&s.id),
            Self::Station(s) => Some(&s.id),
            Self::Other(_) => None,
        }
    }

    /// Name of the place, if any.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Stop(s) => s.name.as_deref(),
            Self::Station(s) => s.name.as_deref(),
            Self::Other(_) => None,
        }
    }

    /// Coordinates as `(latitude, longitude)`, if known.
    pub fn coordinates(&self) -> Option<(f64, f64)> {
        match self {
            Self::Stop(s) => s.location.as_ref().and_then(coords_of),
            Self::Station(s) => s.location.as_ref().and_then(coords_of),
            Self::Other(_) => None,
        }
    }

    /// Borrow the inner [`Stop`], if this is one.
    pub fn as_stop(&self) -> Option<&Stop> {
        if let Self::Stop(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Borrow the inner [`Station`], if this is one.
    pub fn as_station(&self) -> Option<&Station> {
        if let Self::Station(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Raw JSON for unknown values, else `None`.
    pub fn as_other_value(&self) -> Option<&Value> {
        if let Self::Other(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl Place {
    /// ID of the underlying object, if any.
    pub fn id(&self) -> Option<&str> {
        match self {
            Self::Stop(s) => Some(&s.id),
            Self::Station(s) => Some(&s.id),
            Self::Location(l) => l.id.as_deref(),
            Self::Other(_) => None,
        }
    }

    /// Name of the place, if any.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Stop(s) => s.name.as_deref(),
            Self::Station(s) => s.name.as_deref(),
            Self::Location(l) => l.name.as_deref(),
            Self::Other(_) => None,
        }
    }

    /// Coordinates as `(latitude, longitude)`, if known.
    pub fn coordinates(&self) -> Option<(f64, f64)> {
        match self {
            Self::Stop(s) => s.location.as_ref().and_then(coords_of),
            Self::Station(s) => s.location.as_ref().and_then(coords_of),
            Self::Location(l) => coords_of(l),
            Self::Other(_) => None,
        }
    }

    /// Borrow the inner [`Stop`], if this is one.
    pub fn as_stop(&self) -> Option<&Stop> {
        if let Self::Stop(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Borrow the inner [`Station`], if this is one.
    pub fn as_station(&self) -> Option<&Station> {
        if let Self::Station(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Borrow the inner [`Location`], if this is one.
    pub fn as_location(&self) -> Option<&Location> {
        if let Self::Location(l) = self {
            Some(l)
        } else {
            None
        }
    }

    /// Raw JSON for unknown values, else `None`.
    pub fn as_other_value(&self) -> Option<&Value> {
        if let Self::Other(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

fn coords_of(loc: &Location) -> Option<(f64, f64)> {
    match (loc.latitude, loc.longitude) {
        (Some(lat), Some(lon)) => Some((lat, lon)),
        _ => None,
    }
}
