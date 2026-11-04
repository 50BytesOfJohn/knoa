//! Fetching a bookmarked page and turning it into evidence: metadata, exact fields from
//! structured data and site APIs, readable text and a cached preview image.
//!
//! Plain async code on tokio, with no GPUI dependency (ADR 0006).

mod access;
mod html;
mod http;
mod image;
mod jsonld;
mod sites;

use std::path::PathBuf;

use knoa_core::{Fields, Kind, Page};
use serde::Serialize;
use serde_json::Value;
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("network error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("not a valid web address")]
    BadUrl,
}

impl Error {
    /// Network trouble that may go away; worth a retry later.
    pub fn is_transient(&self) -> bool {
        matches!(self, Error::Http(err) if err.is_timeout() || err.is_connect() || err.is_request())
    }
}

/// How much the fetch could read. Decided here, not by the model: models claim "good"
/// for pages they couldn't read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Quality {
    Good,
    Partial,
    TitleOnly,
}

/// Characters of page text the AI sees. Longer text didn't change kind, fields, tags or
/// folders in the enrichment research; it only cost more.
const PROMPT_TEXT_CHARS: usize = 4_000;
/// Characters of page text kept locally for search.
const SEARCH_TEXT_CHARS: usize = 100_000;

#[derive(Serialize)]
struct Evidence<'a> {
    url: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    landed_on: Option<&'a str>,
    quality: Quality,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    site_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    og_type: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    structured_data: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    media: Option<&'a html::Media>,
    #[serde(skip_serializing_if = "Fields::is_empty")]
    known_fields: &'a Fields,
    #[serde(skip_serializing_if = "str::is_empty")]
    text: &'a str,
}

pub struct Fetcher {
    http: http::Http,
    image_cache: PathBuf,
}

impl Fetcher {
    pub fn new(image_cache: PathBuf) -> Result<Fetcher, Error> {
        Ok(Fetcher {
            http: http::Http::new()?,
            image_cache,
        })
    }

    /// Fetches `url` and extracts everything Knoa can learn without the AI.
    ///
    /// Unreadable pages are not errors: they come back as title-only evidence with a note
    /// saying why, which is what the AI needs to infer the bookmark from its URL and title.
    pub async fn fetch(&self, url: &str) -> Result<Page, Error> {
        let url = Url::parse(url).map_err(|_| Error::BadUrl)?;
        if access::is_private(&url) {
            return Ok(title_only(&url, "private network address, not fetched"));
        }
        let site = sites::extract(&self.http, &url).await.unwrap_or_default();
        let response = self.http.page(&url).await?;
        if !response.status.is_success() {
            let note = format!("the site answered HTTP {}", response.status.as_u16());
            let mut page = self.page_from_site(&url, site).await;
            page.evidence = evidence_json(&Evidence {
                note: Some(&note),
                ..bare(&url, Quality::TitleOnly, &page.fields)
            });
            return Ok(page);
        }
        if response.content_type.starts_with("image/") {
            let image = image::download(&self.http, &url, &self.image_cache).await;
            let fields = Fields::new();
            let evidence = evidence_json(&Evidence {
                note: Some("the link is an image file"),
                ..bare(&url, Quality::Partial, &fields)
            });
            return Ok(Page {
                image,
                kind: Some(Kind::Image),
                evidence,
                ..Page::default()
            });
        }
        if !response.content_type.contains("html") {
            let note = format!("a {} file, no text extracted", response.content_type);
            let fields = Fields::new();
            return Ok(Page {
                evidence: evidence_json(&Evidence {
                    note: Some(&note),
                    ..bare(&url, Quality::TitleOnly, &fields)
                }),
                ..Page::default()
            });
        }

        let body = String::from_utf8_lossy(&response.body).into_owned();
        let meta = tokio::task::spawn_blocking(move || html::parse(&body))
            .await
            .unwrap_or_default();
        Ok(self.page_from_html(&url, &response.url, meta, site).await)
    }

    async fn page_from_html(
        &self,
        url: &Url,
        landed: &Url,
        meta: html::HtmlMeta,
        site: sites::SiteFacts,
    ) -> Page {
        let json_ld = jsonld::compact(&meta.json_ld);
        let (mut kind, mut fields) = jsonld::structured(&json_ld).unzip();
        if site.kind.is_some() {
            kind = site.kind;
            fields = Some(site.fields);
        }
        let mut fields = fields.unwrap_or_default();
        let page_duration = meta.duration.as_ref().and_then(|d| {
            Kind::Video
                .field("duration")?
                .coerce(&Value::from(d.as_str()))
        });
        if kind == Some(Kind::Video)
            && !fields.contains_key("duration")
            && let Some(minutes) = page_duration
        {
            fields.insert("duration".into(), minutes);
        }

        let wall = access::wall(url, landed, meta.title.as_deref(), &meta.text);
        let quality = match (wall, meta.text.len()) {
            (Some(_), _) | (None, 0) => Quality::TitleOnly,
            (None, n) if n < 500 => Quality::Partial,
            _ => Quality::Good,
        };
        let readable = wall.is_none();
        let description = site.description.or(meta.description).filter(|_| readable);
        let image_url = site
            .image
            .or(meta.image)
            .filter(|_| readable)
            .and_then(|src| landed.join(&src).ok());
        let image = match image_url {
            Some(src) => image::download(&self.http, &src, &self.image_cache).await,
            None => None,
        };
        let prompt_text: String = if readable {
            meta.text.chars().take(PROMPT_TEXT_CHARS).collect()
        } else {
            String::new()
        };
        let evidence = evidence_json(&Evidence {
            landed_on: (landed != url).then_some(landed.as_str()),
            note: wall,
            page_title: meta.title.as_deref().or(site.title.as_deref()),
            description: description.as_deref(),
            site_name: meta.site_name.as_deref(),
            og_type: meta.og_type.as_deref(),
            language: meta.language.as_deref(),
            structured_data: if readable { json_ld } else { Vec::new() },
            media: Some(&meta.media).filter(|m| readable && !m.is_empty()),
            text: &prompt_text,
            ..bare(url, quality, &fields)
        });
        Page {
            title: meta.title.clone().or_else(|| site.title.clone()),
            image,
            text: if readable {
                meta.text.chars().take(SEARCH_TEXT_CHARS).collect()
            } else {
                String::new()
            },
            description,
            kind,
            fields,
            evidence,
        }
    }

    /// The page itself was refused, but a site API may still know the bookmark.
    async fn page_from_site(&self, url: &Url, site: sites::SiteFacts) -> Page {
        let image = match site.image.and_then(|src| url.join(&src).ok()) {
            Some(src) => image::download(&self.http, &src, &self.image_cache).await,
            None => None,
        };
        Page {
            title: site.title,
            image,
            description: site.description,
            kind: site.kind,
            fields: site.fields,
            ..Page::default()
        }
    }
}

fn bare<'a>(url: &'a Url, quality: Quality, fields: &'a Fields) -> Evidence<'a> {
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

fn title_only(url: &Url, note: &str) -> Page {
    let fields = Fields::new();
    Page {
        evidence: evidence_json(&Evidence {
            note: Some(note),
            ..bare(url, Quality::TitleOnly, &fields)
        }),
        ..Page::default()
    }
}

fn evidence_json(evidence: &Evidence<'_>) -> String {
    serde_json::to_string(evidence).unwrap_or_default()
}
