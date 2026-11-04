//! The answer's JSON schema. It is generic (fields as name/value pairs) because a union of
//! per-kind objects is rejected by some providers and grows with every kind.

use knoa_core::Kind;
use serde_json::{Value, json};

use super::FolderInput;

/// `screening` adds `fits` and `fit_reason`, the answer to the workspace's guideline.
pub fn answer_schema(folders: Option<&[FolderInput]>, screening: bool) -> Value {
    let pair = json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["name", "value"],
        "properties": { "name": { "type": "string" }, "value": { "type": "string" } },
    });
    let kinds: Vec<&str> = Kind::ALL.iter().map(|k| k.slug()).collect();
    let mut properties = json!({
        "kind": { "type": "string", "enum": kinds },
        "summary": { "type": "string" },
        "fields": { "type": "array", "items": pair },
        "extras": { "type": "array", "items": pair },
        "tags": { "type": "array", "items": { "type": "string" } },
    });
    let mut required = vec!["kind", "summary", "fields", "extras", "tags"];
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
