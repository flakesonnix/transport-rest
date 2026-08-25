//! GeoJSON shapes for trips/legs (`polylines=true`).

use serde::{Deserialize, Serialize};

use super::place::LocationResult;

/// GeoJSON Point geometry; `coordinates` is `[longitude, latitude]`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GeometryPoint {
    /// `[longitude, latitude]` pair.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub coordinates: Vec<f64>,
}

/// GeoJSON Feature; the properties describe the stop at this position
/// (may be an empty object for pure shape points).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PolylineFeature {
    /// The stop/location at this position, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<LocationResult>,
    /// Point geometry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geometry: Option<GeometryPoint>,
}

/// GeoJSON FeatureCollection describing a trip or leg shape.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Polyline {
    /// Ordered points of the shape.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<PolylineFeature>,
}
