//! Validating the model's answer. Strict schemas aren't a guarantee, so everything is
//! checked and coerced here, never trusted.

use knoa_core::{Fields, FolderId, Kind, MAX_AI_TAGS, normalize_tag};
use serde::Deserialize;
use serde_json::Value;

use crate::Error;

const MAX_SUMMARY_CHARS: usize = 400;
const MAX_EXTRAS: usize = 12;

#[derive(Deserialize)]
struct Raw {
    kind: String,
    summary: String,
    fields: Vec<Pair>,
    extras: Vec<Pair>,
    tags: Vec<String>,
    #[serde(default)]
    folders: Vec<String>,
}

#[derive(Deserialize)]
struct Pair {
    name: String,
    value: String,
}

pub struct Answer {
    pub kind: Kind,
    pub summary: Option<String>,
    pub fields: Fields,
    pub extras: Vec<(String, String)>,
    pub tags: Vec<String>,
    pub folders: Vec<FolderId>,
}

/// Parses and cleans the answer. `kind_override` is a kind the page itself declared, which
/// wins over the model's pick.
pub fn parse(
    json: Value,
    kind_override: Option<Kind>,
    folder_ids: &[FolderId],
) -> Result<Answer, Error> {
    let raw: Raw = serde_json::from_value(json)
        .map_err(|err| Error::BadAnswer(format!("answer doesn't match the schema: {err}")))?;
    let kind = kind_override
        .or_else(|| Kind::parse(&raw.kind))
        .unwrap_or(Kind::Other);

    let mut fields = Fields::new();
    let mut extras = Vec::new();
    // Models sometimes put a kind's field in extras, or repeat it there.
    for pair in raw.fields.into_iter().chain(raw.extras) {
        let name = pair.name.trim().to_lowercase().replace(' ', "_");
        match kind.field(&name) {
            Some(def) => {
                if let Some(value) = def.coerce(&Value::from(pair.value)) {
                    fields.entry(name).or_insert(value);
                }
            }
            // A fact under a name the kind doesn't have is still a fact.
            None => extras.push((pair.name.trim().to_owned(), pair.value.trim().to_owned())),
        }
    }
    extras.retain(|(name, value)| {
        !name.is_empty() && !value.is_empty() && !value.starts_with("http") && value.len() <= 200
    });
    extras.truncate(MAX_EXTRAS);

    let summary = raw.summary.trim();
    let summary = (!summary.is_empty()).then(|| summary.chars().take(MAX_SUMMARY_CHARS).collect());

    let folders = raw
        .folders
        .iter()
        .filter_map(|id| id.parse().ok().map(FolderId))
        .filter(|id| folder_ids.contains(id))
        .take(2)
        .collect();

    Ok(Answer {
        kind,
        summary,
        fields,
        extras,
        tags: clean_tags(&raw.tags, kind),
        folders,
    })
}

/// Normalized, deduplicated, capped, and without tags that only repeat the kind.
fn clean_tags(tags: &[String], kind: Kind) -> Vec<String> {
    let kind_words = [
        kind.slug().to_owned(),
        kind.label().to_lowercase(),
        kind.plural().to_lowercase(),
    ];
    let mut clean: Vec<String> = Vec::new();
    for tag in tags {
        let tag = normalize_tag(tag);
        if !tag.is_empty() && tag.len() <= 40 && !kind_words.contains(&tag) && !clean.contains(&tag)
        {
            clean.push(tag);
        }
    }
    clean.truncate(MAX_AI_TAGS);
    clean
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn answer_is_coerced_and_cleaned() {
        let answer = parse(
            json!({
                "kind": "movie",
                "summary": " A noir classic. ",
                "fields": [{"name": "runtime", "value": "1h 44m"}, {"name": "studio", "value": "London Films"}],
                "extras": [{"name": "poster", "value": "https://x/y.jpg"}],
                "tags": ["Film Noir", "movie", "film-noir", "Vienna"],
                "folders": ["7", "99"]
            }),
            None,
            &[FolderId(7)],
        )
        .unwrap();
        assert_eq!(answer.kind, Kind::Movie);
        assert_eq!(answer.summary.as_deref(), Some("A noir classic."));
        assert_eq!(Value::Object(answer.fields), json!({"runtime": 104}));
        assert_eq!(
            answer.extras,
            [("studio".to_owned(), "London Films".to_owned())]
        );
        assert_eq!(answer.tags, ["film-noir", "vienna"]);
        assert_eq!(answer.folders, [FolderId(7)]);
    }
}
