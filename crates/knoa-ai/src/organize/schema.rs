//! The answer's JSON schema, built for the one kind already picked. A single schema for
//! every kind would need a union of per-kind objects, which some providers reject and
//! which grows with every kind.

use knoa_core::Kind;
use serde_json::{Map, Value, json};

use super::FolderInput;

/// `screening` adds `fits` and `fit_reason`, the answer to the workspace's guideline.
pub fn answer_schema(kind: Kind, folders: Option<&[FolderInput]>, screening: bool) -> Value {
    let pair = json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["name", "value"],
        "properties": { "name": { "type": "string" }, "value": { "type": "string" } },
    });
    let mut properties = json!({
        "summary": { "type": "string" },
        "fields": fields_schema(kind),
        "extras": { "type": "array", "items": pair },
        "tags": { "type": "array", "items": { "type": "string" } },
    });
    let mut required = vec!["summary", "fields", "extras", "tags"];
    if let Some(folders) = folders.filter(|f| !f.is_empty()) {
        let ids: Vec<String> = folders.iter().map(|f| f.id.0.to_string()).collect();
        properties["folders"] =
            json!({ "type": "array", "items": { "type": "string", "enum": ids } });
        required.push("folders");
    }
    if screening {
        properties["fits"] = json!({ "type": "boolean" });
        properties["fit_reason"] = json!({ "type": "string" });
        required.extend(["fits", "fit_reason"]);
    }
    json!({ "type": "object", "additionalProperties": false, "required": required, "properties": properties })
}

/// Every field is a string or null: strict schemas need every property required, and
/// code coerces the strings to the field's type, which models get wrong either way.
fn fields_schema(kind: Kind) -> Value {
    let fields = &kind.definition().fields;
    let properties: Map<String, Value> = fields
        .iter()
        .map(|f| {
            let schema = json!({ "type": ["string", "null"], "description": f.description });
            (f.name.clone(), schema)
        })
        .collect();
    let names: Vec<&str> = fields.iter().map(|f| f.name.as_str()).collect();
    json!({ "type": "object", "additionalProperties": false, "required": names, "properties": properties })
}
