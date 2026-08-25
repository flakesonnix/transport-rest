//! Internal utilities: URL encoding and tolerant JSON deserialization.

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

/// Characters that must be escaped inside a single path segment.
const PATH_SEGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'?')
    .add(b'{')
    .add(b'}')
    .add(b'%')
    .add(b'/');

/// Percent-encode a value so it can be used as a single path segment.
///
/// Trip IDs and refresh tokens regularly contain `/`, `:` and other
/// characters that must not appear unescaped in a path segment.
pub(crate) fn encode_path_segment(value: &str) -> String {
    utf8_percent_encode(value, PATH_SEGMENT).to_string()
}

/// Module with serde helpers shared by the generated/handwritten models.
///
/// All helpers follow the library's compatibility rules: accept `null`,
/// accept absent fields (via `#[serde(default)]` on the field), and accept
/// numerically interchangeable JSON numbers (`1` vs `1.0`).
pub mod de {
    use serde::{Deserialize, Deserializer};

    /// `Option<i64>` from an integer *or* float *or* null.
    ///
    /// Upstream declares delays both as integers and floats across profiles.
    pub fn opt_i64_lenient<'de, D>(de: D) -> Result<Option<i64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Option::<serde_json::Number>::deserialize(de)?;
        Ok(v.and_then(|n| n.as_i64().or_else(|| n.as_f64().map(|f| f.round() as i64))))
    }

    /// `Option<i64>` from an integer *or* float *or* null (same as
    /// [`opt_i64_lenient`], separate name for documentation clarity).
    pub fn opt_epoch_seconds<'de, D>(de: D) -> Result<Option<i64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        opt_i64_lenient(de)
    }

    /// `i64` from an integer or float, defaulting to `0` on absence/null.
    pub fn i64_default_zero<'de, D>(de: D) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(opt_i64_lenient(de)?.unwrap_or(0))
    }

    /// `Option<f64>` from any JSON number or null.
    pub fn opt_f64_lenient<'de, D>(de: D) -> Result<Option<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Option::<serde_json::Number>::deserialize(de)?;
        Ok(v.and_then(|n| n.as_f64()))
    }
}

/// Serde module for `Option<i64>` fields that may arrive as int, float or null
/// and are serialized back as plain integers.
pub mod lenient_i64 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub(crate) fn serialize<S: Serializer>(v: &Option<i64>, s: S) -> Result<S::Ok, S::Error> {
        match v {
            Some(v) => s.serialize_some(v),
            None => s.serialize_none(),
        }
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<i64>, D::Error> {
        super::de::opt_i64_lenient(d)
    }
}
