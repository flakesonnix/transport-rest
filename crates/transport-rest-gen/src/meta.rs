//! Emits the validated endpoint/provider metadata as canonical JSON.
//!
//! Native client implementations (TypeScript, Go, C#, Java) read this file
//! so their request layer stays in sync with the schema without duplicating
//! parameter tables.

use crate::schema::Ir;
use serde_json::json;

pub fn emit(ir: &Ir) -> String {
    let endpoints = ir
        .rest
        .endpoints
        .iter()
        .map(|e| {
            json!({
                "id": e.id,
                "method": e.method,
                "path": e.path,
                "doc": e.doc,
                "capability": e.capability,
                "responseType": e.response_type,
                "params": e.params.iter().map(|p| json!({
                    "name": p.name,
                    "json": p.json.clone().unwrap_or_else(|| p.name.clone()),
                    "in": p.in_,
                    "type": p.type_,
                    "required": p.required,
                    "default": p.default,
                    "doc": p.doc,
                    "group": p.group,
                    "dbOnly": p.db_only,
                })).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();

    let value = json!({
        "$generated-by": "transport-rest-gen",
        "schemaVersion": ir.schema_version,
        "providers": ir.providers.iter().map(|(k, v)| json!({
            "id": k,
            "baseUrl": v.base_url,
            "region": v.region,
        })).collect::<Vec<_>>(),
        "productKeys": {
            "db": ir.rest.product_keys_db,
            "bvg": ir.rest.product_keys_bvg,
        },
        "endpoints": endpoints,
    });

    #[allow(clippy::expect_used)] // value contains no non-string keys; cannot fail
    let mut out = serde_json::to_string_pretty(&value).expect("static JSON serialization");
    out.push('\n');
    out
}
