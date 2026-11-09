//! Validating the model's answer. Strict schemas aren't a guarantee, so everything is
//! checked and coerced here, never trusted.

use knoa_core::{Fields, FolderId, Kind, MAX_AI_TAGS, MAX_TAG_CHARS, TagList, normalize_tag};
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::Error;

const MAX_SUMMARY_CHARS: usize = 400;
const MAX_EXTRAS: usize = 12;

#[derive(Deserialize)]
struct Raw {
    summary: String,
    /// The kind's fields by name, null when unknown.
    fields: Map<String, Value>,
    extras: Vec<Pair>,
    tags: Vec<String>,
    #[serde(default)]
    folders: Vec<String>,
    /// Missing when the workspace doesn't screen, which means it fits.
    #[serde(default)]
    fits: Option<bool>,
    #[serde(default)]
    fit_reason: String,
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
    /// Why the bookmark doesn't fit the workspace's guideline, when it doesn't.
    pub set_aside: Option<String>,
}

/// Parses and cleans the answer for a bookmark of `kind`.
pub fn parse(
    json: Value,
    kind: Kind,
    folder_ids: &[FolderId],
    tag_list: &TagList,
) -> Result<Answer, Error> {
    let raw: Raw = serde_json::from_value(json)
        .map_err(|err| Error::BadAnswer(format!("answer doesn't match the schema: {err}")))?;

    let mut fields = Fields::new();
    for (name, value) in &raw.fields {
        if let Some(value) = kind.field(name).and_then(|def| def.coerce(value)) {
            fields.insert(name.clone(), value);
        }
    }
    let mut extras = Vec::new();
    for pair in raw.extras {
        let name = pair.name.trim().to_lowercase().replace(' ', "_");
        match kind.field(&name) {
            // Models sometimes put a kind's field in extras too.
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

    let set_aside = (raw.fits == Some(false)).then(|| {
        let reason = raw.fit_reason.trim();
        if reason.is_empty() {
            "It doesn't fit this workspace's guideline.".to_owned()
        } else {
            reason.chars().take(MAX_SUMMARY_CHARS).collect()
        }
    });

    Ok(Answer {
        kind,
        summary,
        fields,
        extras,
        tags: clean_tags(&raw.tags, kind, tag_list),
        folders,
        set_aside,
    })
}

/// Normalized, deduplicated and capped, keeping only the tags `keeps` allows.
fn clean_tags(tags: &[String], kind: Kind, tag_list: &TagList) -> Vec<String> {
    let mut clean: Vec<String> = Vec::new();
    for tag in tags {
        let tag = normalize_tag(tag);
        if keeps(&tag, kind, tag_list) && !clean.contains(&tag) {
            clean.push(tag);
        }
    }
    clean.truncate(MAX_AI_TAGS);
    clean
}

/// A listed tag always stays, even one that names the kind. When the AI is held to the
/// list, nothing else does; otherwise a tag stays unless it's too long or only repeats
/// the kind.
fn keeps(tag: &str, kind: Kind, tag_list: &TagList) -> bool {
    if tag.is_empty() || tag.len() > MAX_TAG_CHARS {
        return false;
    }
    if tag_list.contains(tag) {
        return true;
    }
    if tag_list.restricts() {
        return false;
    }
    let kind_words = [
        kind.slug().to_owned(),
        kind.label().to_lowercase(),
        kind.plural().to_lowercase(),
    ];
    !kind_words.iter().any(|word| word == tag)
}

#[cfg(test)]
mod tests {
    use knoa_core::ListedTag;
    use serde_json::json;

    use super::*;

    #[test]
    fn answer_is_coerced_and_cleaned() {
        let answer = parse(
            json!({
                "summary": " A noir classic. ",
                "fields": {"runtime": "1h 44m", "year": null},
                "extras": [{"name": "poster", "value": "https://x/y.jpg"}, {"name": "studio", "value": "London Films"}, {"name": "Year", "value": "1949"}],
                "tags": ["Film Noir", "movie", "film-noir", "Vienna"],
                "folders": ["7", "99"]
            }),
            Kind::Movie,
            &[FolderId(7)],
            &TagList::default(),
        )
        .unwrap();
        assert_eq!(answer.kind, Kind::Movie);
        assert_eq!(answer.summary.as_deref(), Some("A noir classic."));
        assert_eq!(
            Value::Object(answer.fields),
            json!({"runtime": 104, "year": 1949})
        );
        assert_eq!(
            answer.extras,
            [("studio".to_owned(), "London Films".to_owned())]
        );
        assert_eq!(answer.tags, ["film-noir", "vienna"]);
        assert_eq!(answer.folders, [FolderId(7)]);
    }

    #[test]
    fn only_an_explicit_misfit_is_set_aside() {
        let answer = |extra: Value| {
            let mut json = json!({"summary": "", "fields": {}, "extras": [], "tags": []});
            json.as_object_mut()
                .unwrap()
                .extend(extra.as_object().unwrap().clone());
            parse(json, Kind::Article, &[], &TagList::default())
                .unwrap()
                .set_aside
        };
        assert_eq!(answer(json!({})), None);
        assert_eq!(answer(json!({"fits": true, "fit_reason": "work"})), None);
        assert_eq!(
            answer(json!({"fits": false, "fit_reason": " A recipe, not work. "})).as_deref(),
            Some("A recipe, not work.")
        );
    }

    #[test]
    fn a_list_the_ai_is_held_to_keeps_only_listed_tags() {
        let tag_list = TagList {
            tags: ["article", "rust"]
                .map(|name| ListedTag {
                    name: name.into(),
                    description: String::new(),
                })
                .into(),
            only_these: true,
        };
        let json = json!({"summary": "", "fields": {}, "extras": [], "tags": ["Rust", "compilers", "article"]});
        let answer = parse(json, Kind::Article, &[], &tag_list).unwrap();
        assert_eq!(answer.tags, ["rust", "article"]);
    }
}
