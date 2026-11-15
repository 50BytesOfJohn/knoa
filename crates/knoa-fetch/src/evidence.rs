//! The evidence: everything a fetch learned, as the JSON the AI reads.

use knoa_core::Fields;
use serde::Serialize;
use serde_json::Value;
use url::Url;

use crate::html;

/// How much the fetch could read. Decided here, not by the model: models claim "good"
/// for pages they couldn't read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Quality {
    Good,
    Partial,
    TitleOnly,
}

#[derive(Serialize)]
pub struct Evidence<'a> {
    pub url: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub landed_on: Option<&'a str>,
    pub quality: Quality,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_type: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub structured_data: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<&'a html::Media>,
    #[serde(skip_serializing_if = "Fields::is_empty")]
    pub known_fields: &'a Fields,
    #[serde(skip_serializing_if = "str::is_empty")]
    pub text: &'a str,
}

pub fn bare<'a>(url: &'a Url, quality: Quality, fields: &'a Fields) -> Evidence<'a> {
    Evidence {
        url: url.as_str(),
        landed_on: None,
        quality,
        note: None,
        page_title: None,
        description: None,
        site_name: None,
        og_type: None,
        language: None,
        structured_data: Vec::new(),
        media: None,
        known_fields: fields,
        text: "",
    }
}

pub fn evidence_json(evidence: &Evidence<'_>) -> String {
    serde_json::to_string(evidence).unwrap_or_default()
}
