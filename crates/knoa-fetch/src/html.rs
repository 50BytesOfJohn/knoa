//! Metadata and readable text from an HTML page.

use std::sync::LazyLock;

use scraper::{ElementRef, Html, Node, Selector};
use serde::Serialize;
use serde_json::Value;

use crate::jsonld;

/// What the page says about itself.
#[derive(Debug, Default)]
pub struct HtmlMeta {
    pub title: Option<String>,
    pub description: Option<String>,
    pub site_name: Option<String>,
    pub og_type: Option<String>,
    pub image: Option<String>,
    pub language: Option<String>,
    /// `itemprop="duration"`, which YouTube and many video sites set as ISO 8601.
    pub duration: Option<String>,
    pub json_ld: Vec<Value>,
    pub media: Media,
    pub text: String,
}

/// Video players on the page. Galleries tagged `og:type=article` often embed several, and
/// without this the AI never learns the page is about video.
#[derive(Debug, Default, Serialize)]
pub struct Media {
    pub videos: usize,
    pub video_iframes: usize,
}

impl Media {
    pub fn is_empty(&self) -> bool {
        self.videos == 0 && self.video_iframes == 0
    }
}

fn selector(css: &str) -> Selector {
    Selector::parse(css).expect("selectors are literals")
}

static TITLE: LazyLock<Selector> = LazyLock::new(|| selector("title"));
static META: LazyLock<Selector> = LazyLock::new(|| selector("meta"));
static HTML: LazyLock<Selector> = LazyLock::new(|| selector("html"));
static JSON_LD: LazyLock<Selector> =
    LazyLock::new(|| selector(r#"script[type="application/ld+json"]"#));
static BODY: LazyLock<Selector> = LazyLock::new(|| selector("body"));
static VIDEO: LazyLock<Selector> = LazyLock::new(|| selector("video"));
static IFRAME: LazyLock<Selector> = LazyLock::new(|| selector("iframe"));

/// Elements whose text is navigation or code, not the page's content.
const SKIPPED: &[&str] = &[
    "script", "style", "noscript", "nav", "footer", "header", "svg", "template", "iframe", "form",
];

pub fn parse(html: &str) -> HtmlMeta {
    let doc = Html::parse_document(html);
    let mut meta = HtmlMeta {
        title: doc
            .select(&TITLE)
            .next()
            .map(|t| collapse(&t.text().collect::<String>())),
        language: doc
            .select(&HTML)
            .next()
            .and_then(|h| h.attr("lang"))
            .map(str::to_owned),
        ..HtmlMeta::default()
    };
    read_meta_tags(&doc, &mut meta);
    meta.json_ld = doc
        .select(&JSON_LD)
        .filter_map(|script| serde_json::from_str::<Value>(&script.text().collect::<String>()).ok())
        .collect();
    // Many video pages set no `og:image`, but their player still names a still.
    meta.image = meta
        .image
        .or_else(|| jsonld::video_thumbnail(&meta.json_ld))
        .or_else(|| first_poster(&doc));
    meta.media = media(&doc);
    if let Some(body) = doc.select(&BODY).next() {
        let mut text = String::new();
        readable_text(body, &mut text);
        meta.text = collapse(&text);
    }
    meta
}

fn read_meta_tags(doc: &Html, meta: &mut HtmlMeta) {
    for tag in doc.select(&META) {
        let key = tag
            .attr("property")
            .or_else(|| tag.attr("name"))
            .or_else(|| tag.attr("itemprop"));
        let (Some(key), Some(content)) = (key, tag.attr("content")) else {
            continue;
        };
        let content = collapse(content);
        if content.is_empty() {
            continue;
        }
        let key = key.to_ascii_lowercase();
        if key == "og:title" {
            // Usually cleaner than `<title>`, which often carries the site name.
            meta.title = Some(content);
            continue;
        }
        let slot = match key.as_str() {
            "og:description" | "description" | "twitter:description" => &mut meta.description,
            "og:site_name" => &mut meta.site_name,
            "og:type" => &mut meta.og_type,
            "og:image" | "og:image:url" | "twitter:image" => &mut meta.image,
            "duration" => &mut meta.duration,
            _ => continue,
        };
        // The first value wins: `og:` tags come before `twitter:` ones on most pages.
        if slot.is_none() {
            *slot = Some(content);
        }
    }
}

fn first_poster(doc: &Html) -> Option<String> {
    doc.select(&VIDEO)
        .filter_map(|video| video.attr("poster"))
        .map(str::trim)
        .find(|poster| !poster.is_empty())
        .map(str::to_owned)
}

fn media(doc: &Html) -> Media {
    Media {
        videos: doc.select(&VIDEO).count(),
        video_iframes: doc.select(&IFRAME).filter(|f| is_video_iframe(*f)).count(),
    }
}

/// Embedded players ask for picture-in-picture or DRM, or say "player" or "video" in their
/// address; maps, forms and social embeds do neither.
fn is_video_iframe(iframe: ElementRef<'_>) -> bool {
    let allow = iframe.attr("allow").unwrap_or_default();
    let src = iframe.attr("src").unwrap_or_default().to_ascii_lowercase();
    allow.contains("picture-in-picture")
        || allow.contains("encrypted-media")
        || src.contains("player")
        || src.contains("video")
}

fn readable_text(element: ElementRef<'_>, out: &mut String) {
    for child in element.children() {
        match child.value() {
            Node::Text(text) => {
                out.push_str(text);
                out.push(' ');
            }
            Node::Element(el) if !SKIPPED.contains(&el.name()) => {
                if let Some(child) = ElementRef::wrap(child) {
                    readable_text(child, out);
                }
            }
            _ => {}
        }
    }
}

/// Whitespace runs become one space.
pub fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_open_graph_and_skips_navigation_text() {
        let meta = parse(
            r#"<html lang="pl"><head><title>Raw title</title>
            <meta property="og:title" content="Pancakes">
            <meta name="description" content="Fluffy  pancakes">
            <meta property="og:image" content="/img/p.jpg">
            <script type="application/ld+json">{"@type": "Recipe", "totalTime": "PT20M"}</script>
            </head><body><nav>Home About</nav><p>Mix the   flour.</p><script>track()</script></body></html>"#,
        );
        assert_eq!(meta.title.as_deref(), Some("Pancakes"));
        assert_eq!(meta.description.as_deref(), Some("Fluffy pancakes"));
        assert_eq!(meta.image.as_deref(), Some("/img/p.jpg"));
        assert_eq!(meta.language.as_deref(), Some("pl"));
        assert_eq!(meta.text, "Mix the flour.");
        assert_eq!(meta.json_ld.len(), 1);
    }

    #[test]
    fn video_pages_without_og_image_use_the_player_still() {
        let thumbnail = parse(
            r#"<html><head><script type="application/ld+json">
            {"@graph": [{"@type": "VideoObject", "thumbnailUrl": [{"@type": "ImageObject", "url": "/t.jpg"}]}]}
            </script></head><body><video poster="/p.jpg"></video></body></html>"#,
        );
        assert_eq!(thumbnail.image.as_deref(), Some("/t.jpg"));
        let poster = parse(r#"<html><body><video poster=" /p.jpg "></video></body></html>"#);
        assert_eq!(poster.image.as_deref(), Some("/p.jpg"));
    }
}
