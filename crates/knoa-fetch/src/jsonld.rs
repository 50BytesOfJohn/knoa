//! schema.org JSON-LD: compacted for the prompt, and read directly for exact fields.

use knoa_core::{Fields, Kind};
use serde_json::{Map, Value};

/// Keys that add size but nothing the AI needs.
const DROPPED: &[&str] = &[
    "@context",
    "review",
    "comment",
    "potentialAction",
    "breadcrumb",
    "image",
    "logo",
    "thumbnailUrl",
    "url",
    "sameAs",
    "mainEntityOfPage",
    "publisher",
    "isPartOf",
    "interactionStatistic",
    "associatedMedia",
    "recipeInstructions",
    "offers",
];

/// The page's JSON-LD items, flattened out of `@graph` and stripped of bulky keys.
pub fn compact(items: &[Value]) -> Vec<Value> {
    items
        .iter()
        .flat_map(flatten)
        .map(|item| strip(&item))
        .filter(|item| !item.as_object().is_some_and(Map::is_empty))
        .collect()
}

/// The still a `VideoObject` names, read from the raw items since `compact` drops images.
pub fn video_thumbnail(items: &[Value]) -> Option<String> {
    items
        .iter()
        .flat_map(flatten)
        .filter(|item| type_of(item).as_deref() == Some("VideoObject"))
        .find_map(|video| {
            ["thumbnailUrl", "image"]
                .iter()
                .find_map(|key| image_url(video.get(key)?))
        })
}

/// A URL given as a string, a list, or an `ImageObject`.
fn image_url(value: &Value) -> Option<String> {
    match value {
        Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_owned()),
        Value::Array(items) => items.iter().find_map(image_url),
        Value::Object(map) => ["url", "contentUrl"]
            .iter()
            .find_map(|key| image_url(map.get(*key)?)),
        _ => None,
    }
}

fn flatten(item: &Value) -> Vec<Value> {
    match item {
        Value::Array(items) => items.iter().flat_map(flatten).collect(),
        Value::Object(map) => match map.get("@graph") {
            Some(graph) => flatten(graph),
            None => vec![item.clone()],
        },
        _ => Vec::new(),
    }
}

fn strip(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(k, _)| !DROPPED.contains(&k.as_str()))
                .map(|(k, v)| (k.clone(), strip(v)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().take(8).map(strip).collect()),
        Value::String(s) if s.len() > 500 => Value::from(s.chars().take(500).collect::<String>()),
        other => other.clone(),
    }
}

/// A kind and exact fields from the most specific item on the page. Pages embed generic
/// items (a recipe page carries a `VideoObject` for its how-to clip), so a recipe, movie or
/// product wins over a video.
pub fn structured(items: &[Value]) -> Option<(Kind, Fields)> {
    let (kind, item) = items
        .iter()
        .filter_map(|item| Some((kind_of(&type_of(item)?)?, item)))
        .min_by_key(|(kind, _)| usize::from(*kind == Kind::Video))?;
    Some((kind, fields_for(kind, item)))
}

fn kind_of(schema_type: &str) -> Option<Kind> {
    Some(match schema_type {
        "Movie" | "TVSeries" => Kind::Movie,
        "Recipe" => Kind::Recipe,
        "VideoObject" => Kind::Video,
        "Book" => Kind::Book,
        "Product" => Kind::Product,
        "PodcastEpisode" | "PodcastSeries" => Kind::Podcast,
        "SoftwareApplication" | "WebApplication" | "MobileApplication" => Kind::App,
        "Restaurant" | "LocalBusiness" | "Place" | "Hotel" | "Museum" | "TouristAttraction" => {
            Kind::Place
        }
        _ => return None,
    })
}

fn type_of(item: &Value) -> Option<String> {
    match item.get("@type")? {
        Value::String(s) => Some(s.clone()),
        Value::Array(types) => types.iter().find_map(Value::as_str).map(str::to_owned),
        _ => None,
    }
}

fn fields_for(kind: Kind, item: &Value) -> Fields {
    let pairs: &[(&str, &str)] = match kind {
        Kind::Movie => &[
            ("year", "datePublished"),
            ("runtime", "duration"),
            ("director", "director"),
            ("cast", "actor"),
            ("genre", "genre"),
        ],
        Kind::Recipe => &[
            ("total_time", "totalTime"),
            ("cuisine", "recipeCuisine"),
            ("servings", "recipeYield"),
        ],
        Kind::Video => &[("duration", "duration"), ("published", "uploadDate")],
        Kind::Book => &[("author", "author"), ("pages", "numberOfPages")],
        Kind::Product => &[("brand", "brand")],
        Kind::Podcast => &[("duration", "duration"), ("published", "datePublished")],
        Kind::Place => &[
            ("city", "address.addressLocality"),
            ("country", "address.addressCountry"),
            ("address", "address.streetAddress"),
        ],
        _ => &[],
    };
    let mut fields = Fields::new();
    for (field, key) in pairs {
        let Some(def) = kind.field(field) else {
            continue;
        };
        let Some(raw) = lookup(item, key) else {
            continue;
        };
        let raw = names(raw);
        // `datePublished` → year for movies.
        let raw = if *field == "year" {
            Value::from(raw.as_str().and_then(|d| d.get(..4)).unwrap_or_default())
        } else {
            raw
        };
        if let Some(value) = def.coerce(&raw) {
            fields.insert((*field).to_owned(), value);
        }
    }
    fields
}

fn lookup<'a>(item: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(item, |value, key| value.get(key))
}

/// People and organizations come as `{"@type": "Person", "name": ...}`; keep the names.
fn names(value: &Value) -> Value {
    match value {
        Value::Object(map) => map.get("name").cloned().unwrap_or(Value::Null),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(names)
                .filter(|v| !v.is_null())
                .take(5)
                .collect(),
        ),
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn movie_fields_come_from_json_ld() {
        let items = [
            json!({"@graph": [{"@type": "Movie", "name": "The Third Man", "datePublished": "1949-09-02",
            "duration": "PT1H44M", "director": [{"@type": "Person", "name": "Carol Reed"}], "image": "x.jpg"}]}),
        ];
        let (kind, fields) = structured(&compact(&items)).unwrap();
        assert_eq!(kind, Kind::Movie);
        assert_eq!(
            Value::Object(fields),
            json!({"year": 1949, "runtime": 104, "director": ["Carol Reed"]})
        );
    }
}
