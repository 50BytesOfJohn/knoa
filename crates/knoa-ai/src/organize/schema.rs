//! The answer's JSON schema. It is generic (fields as name/value pairs) because a union of
//! per-kind objects is rejected by some providers and grows with every kind.

use knoa_core::Kind;
use serde_json::{Value, json};

use super::FolderInput;

pub fn answer_schema(folders: Option<&[FolderInput]>) -> Value {
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
    json!({ "type": "object", "additionalProperties": false, "required": required, "properties": properties })
}
