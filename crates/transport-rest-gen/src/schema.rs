//! Schema parser: loads and merges the internal representation (IR) from
//! `schema/*.json` into one validated model.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // IR is a faithful mirror of schema/*.json
pub struct Ir {
    #[serde(rename = "$schema-version")]
    pub schema_version: u32,
    pub name: String,
    #[serde(default)]
    pub providers: BTreeMap<String, Provider>,
    #[serde(default)]
    pub capabilities: Option<serde_json::Value>,
    #[serde(flatten)]
    pub rest: Rest,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Provider {
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    pub region: String,
}

/// Everything that may be spread across multiple schema files.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
pub struct Rest {
    #[serde(default)]
    pub enums: BTreeMap<String, EnumDef>,
    #[serde(default)]
    pub aliases: BTreeMap<String, AliasDef>,
    #[serde(default)]
    pub unions: BTreeMap<String, UnionDef>,
    #[serde(default)]
    pub models: BTreeMap<String, ModelDef>,
    #[serde(default)]
    pub envelopes: BTreeMap<String, ModelDef>,
    #[serde(default)]
    pub list_responses: BTreeMap<String, String>,
    #[serde(default)]
    pub endpoints: Vec<EndpointDef>,
    #[serde(default, rename = "productKeysDb")]
    pub product_keys_db: Vec<String>,
    #[serde(default, rename = "productKeysBvg")]
    pub product_keys_bvg: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnumDef {
    #[serde(default)]
    pub doc: Option<String>,
    /// True if the enum accepts unknown values at runtime.
    #[serde(default)]
    #[allow(dead_code)]
    pub open: bool,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AliasDef {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub doc: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnionDef {
    /// Name of the discriminator field.
    #[serde(default)]
    #[allow(dead_code)]
    pub tag: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub open: bool,
    #[serde(default)]
    pub doc: Option<String>,
    pub variants: Vec<UnionVariant>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnionVariant {
    pub value: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelDef {
    #[serde(default)]
    pub doc: Option<String>,
    pub fields: Vec<FieldDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FieldDef {
    pub name: String,
    /// Wire name; defaults to `name`.
    pub json: Option<String>,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub opt: bool,
    /// Only supported by the DB instance.
    #[serde(default)]
    #[allow(dead_code)]
    pub db_only: bool,
    #[serde(default)]
    pub doc: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EndpointDef {
    pub id: String,
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub doc: Option<String>,
    #[serde(rename = "responseType")]
    pub response_type: String,
    #[serde(default)]
    pub capability: Option<String>,
    pub params: Vec<ParamDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParamDef {
    pub name: String,
    /// Wire name; defaults to `name`.
    pub json: Option<String>,
    #[serde(rename = "in")]
    pub in_: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    #[serde(default)]
    pub doc: Option<String>,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub db_only: bool,
}

/// Load all `*.json` files from `dir` (sorted by filename) and merge them.
///
/// Later files override earlier ones on a top-level key basis only for
/// scalars; maps are merged entry-wise. This makes the split across
/// `base.json` / `types.json` / `models-*.json` / `endpoints.json`
/// deterministic.
pub fn load_ir(dir: &Path) -> Result<Ir, String> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| format!("cannot read {}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "json").unwrap_or(false))
        .collect();
    entries.sort();

    let mut merged = serde_json::Map::new();
    for path in entries {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let value: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("invalid JSON in {}: {e}", path.display()))?;
        let obj = value
            .as_object()
            .ok_or_else(|| format!("{}: expected object", path.display()))?;
        for (k, v) in obj {
            match (merged.get_mut(k), k.as_str()) {
                // scalar keys from base.json must not be duplicated
                (_, "$schema-version" | "name" | "title" | "description") => {
                    merged.insert(k.clone(), v.clone());
                }
                (Some(existing @ serde_json::Value::Object(_)), _) => {
                    if let Some(new_obj) = v.as_object() {
                        merge_objects(existing, new_obj);
                    }
                }
                _ => {
                    merged.insert(k.clone(), v.clone());
                }
            }
        }
    }

    let ir: Ir = serde_json::from_value(serde_json::Value::Object(merged))
        .map_err(|e| format!("invalid IR: {e}"))?;
    validate(&ir)?;
    Ok(ir)
}

fn merge_objects(target: &mut serde_json::Value, new: &serde_json::Map<String, serde_json::Value>) {
    if let Some(t) = target.as_object_mut() {
        for (k, v) in new {
            match t.get_mut(k) {
                Some(existing @ serde_json::Value::Object(_)) => {
                    if let Some(inner) = v.as_object() {
                        merge_objects(existing, inner);
                    }
                }
                _ => {
                    t.insert(k.clone(), v.clone());
                }
            }
        }
    }
}

fn validate(ir: &Ir) -> Result<(), String> {
    let mut known_types: Vec<&str> = Vec::new();
    known_types.extend(ir.rest.enums.keys().map(String::as_str));
    known_types.extend(ir.rest.unions.keys().map(String::as_str));
    known_types.extend(ir.rest.models.keys().map(String::as_str));
    known_types.extend(ir.rest.envelopes.keys().map(String::as_str));
    known_types.extend([
        "string",
        "int",
        "float",
        "bool",
        "datetime",
        "LocationFilter",
        "products",
    ]);
    // Alias names are valid type references; their payloads are validated too.
    for name in ir.rest.aliases.keys() {
        known_types.push(name.as_str());
    }
    for alias in ir.rest.aliases.values() {
        check_type(&alias.type_, &known_types)?;
    }
    for model in ir.rest.models.values().chain(ir.rest.envelopes.values()) {
        for field in &model.fields {
            check_type(&field.type_, &known_types)?;
        }
    }
    Ok(())
}

fn check_type(ty: &str, known: &[&str]) -> Result<(), String> {
    if let Some(inner) = ty.strip_prefix("list<").and_then(|s| s.strip_suffix('>')) {
        return check_type(inner, known);
    }
    if let Some(inner) = ty.strip_prefix("map<").and_then(|s| s.strip_suffix('>')) {
        return check_type(inner, known);
    }
    if known.contains(&ty) {
        return Ok(());
    }
    Err(format!("unknown type '{ty}' referenced in schema"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn merges_split_schema_files() {
        let dir = Path::new("../../schema");
        let ir = load_ir(dir).expect("repo schema is valid");
        assert!(ir.rest.models.contains_key("Stop"));
        assert!(ir.rest.envelopes.contains_key("JourneysResponse"));
        assert!(ir.rest.endpoints.iter().any(|e| e.id == "journeys"));
        // providers come from base.json
        assert_eq!(ir.providers.len(), 4);
    }

    #[test]
    fn rejects_unknown_type_references() {
        let dir = std::env::temp_dir().join("trg-bad-schema");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("bad.json"),
            r#"{"$schema-version": 1, "name": "t", "models": {"X": {"fields": [{"name": "a", "type": "NoSuchType"}]}}}"#,
        )
        .unwrap();
        let err = load_ir(&dir).unwrap_err();
        assert!(err.contains("unknown type 'NoSuchType'"), "got {err}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
