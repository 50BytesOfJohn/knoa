//! Extractors for popular sites, where a public API gives exact facts the page hides.

use knoa_core::{Fields, Kind};
use serde::Deserialize;
use serde_json::Value;
use url::Url;

use crate::http::Http;

/// What a site extractor found.
#[derive(Debug, Default)]
pub struct SiteFacts {
    pub kind: Option<Kind>,
    pub fields: Fields,
    pub title: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
}

pub async fn extract(http: &Http, url: &Url) -> Option<SiteFacts> {
    let host = url
        .host_str()?
        .trim_start_matches("www.")
        .trim_start_matches("m.");
    let result = match host {
        "youtube.com" | "youtu.be" | "vimeo.com" => oembed(http, url, host).await,
        "github.com" => github(http, url).await,
        _ => return None,
    };
    result
        .inspect_err(|err| tracing::debug!(%url, "site extractor failed: {err}"))
        .ok()
        .flatten()
}

#[derive(Deserialize)]
struct OEmbed {
    title: Option<String>,
    author_name: Option<String>,
    thumbnail_url: Option<String>,
    duration: Option<f64>,
}

async fn oembed(http: &Http, url: &Url, host: &str) -> Result<Option<SiteFacts>, crate::Error> {
    let endpoint = if host == "vimeo.com" {
        "https://vimeo.com/api/oembed.json"
    } else {
        "https://www.youtube.com/oembed"
    };
    let mut api = Url::parse(endpoint).expect("literal URL");
    api.query_pairs_mut()
        .append_pair("url", url.as_str())
        .append_pair("format", "json");
    let Some(embed) = http.get_json::<OEmbed>(&api).await? else {
        return Ok(None);
    };
    let mut fields = Fields::new();
    if let Some(channel) = embed.author_name {
        fields.insert("channel".into(), Value::from(channel));
    }
    fields.insert(
        "platform".into(),
        Value::from(if host == "vimeo.com" {
            "Vimeo"
        } else {
            "YouTube"
        }),
    );
    if let Some(seconds) = embed.duration {
        fields.insert(
            "duration".into(),
            Value::from((seconds / 60.0).round() as i64),
        );
    }
    Ok(Some(SiteFacts {
        kind: Some(Kind::Video),
        fields,
        title: embed.title,
        description: None,
        image: embed.thumbnail_url,
    }))
}

#[derive(Deserialize)]
struct Repo {
    name: String,
    owner: Owner,
    description: Option<String>,
    language: Option<String>,
    stargazers_count: Option<i64>,
    license: Option<License>,
}

#[derive(Deserialize)]
struct Owner {
    login: String,
}

#[derive(Deserialize)]
struct License {
    spdx_id: Option<String>,
}

/// Repository pages only (`github.com/owner/name`); other GitHub pages go through HTML.
async fn github(http: &Http, url: &Url) -> Result<Option<SiteFacts>, crate::Error> {
    let segments: Vec<&str> = url
        .path_segments()
        .map(|s| s.filter(|p| !p.is_empty()).collect())
        .unwrap_or_default();
    let [owner, name, ..] = segments.as_slice() else {
        return Ok(None);
    };
    if segments.len() > 2 && !matches!(segments[2], "tree" | "blob") {
        return Ok(None);
    }
    let api = Url::parse(&format!("https://api.github.com/repos/{owner}/{name}"))
        .map_err(|_| crate::Error::BadUrl)?;
    let Some(repo) = http.get_json::<Repo>(&api).await? else {
        return Ok(None);
    };
    let mut fields = Fields::new();
    fields.insert("owner".into(), Value::from(repo.owner.login));
    fields.insert("name".into(), Value::from(repo.name));
    if let Some(language) = repo.language {
        fields.insert("language".into(), Value::from(language));
    }
    if let Some(stars) = repo.stargazers_count {
        fields.insert("stars".into(), Value::from(stars));
    }
    if let Some(spdx) = repo
        .license
        .and_then(|l| l.spdx_id)
        .filter(|id| id != "NOASSERTION")
    {
        fields.insert("license".into(), Value::from(spdx));
    }
    Ok(Some(SiteFacts {
        kind: Some(Kind::Repository),
        fields,
        title: None,
        description: repo.description,
        image: None,
    }))
}
