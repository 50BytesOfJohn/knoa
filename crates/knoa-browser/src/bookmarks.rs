//! Parsing Chromium's `Bookmarks` JSON file.

use std::path::Path;

use knoa_core::NewBookmark;
use serde::Deserialize;

use crate::Error;

/// Seconds between 1601-01-01, Chromium's epoch, and the Unix epoch.
const CHROMIUM_EPOCH_OFFSET: i64 = 11_644_473_600;

#[derive(Debug, Clone, PartialEq)]
pub struct BookmarksFile {
    /// Chromium's own checksum of the contents; unchanged checksum, unchanged bookmarks.
    pub checksum: String,
    pub bookmarks: Vec<NewBookmark>,
}

#[derive(Deserialize)]
struct File {
    #[serde(default)]
    checksum: String,
    roots: Roots,
}

#[derive(Deserialize)]
struct Roots {
    bookmark_bar: Option<Node>,
    other: Option<Node>,
    synced: Option<Node>,
}

#[derive(Deserialize)]
struct Node {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    name: String,
    url: Option<String>,
    guid: Option<String>,
    #[serde(default)]
    date_added: String,
    #[serde(default)]
    children: Vec<Node>,
}

/// Reads every web bookmark in the file, with its folder path ("Bookmarks Bar/Recipes").
pub fn read_bookmarks(path: &Path) -> Result<BookmarksFile, Error> {
    let text = std::fs::read_to_string(path).map_err(|err| Error::Io {
        path: path.to_owned(),
        err,
    })?;
    let file: File = serde_json::from_str(&text).map_err(|err| Error::Parse {
        path: path.to_owned(),
        err,
    })?;
    let mut bookmarks = Vec::new();
    for root in [file.roots.bookmark_bar, file.roots.other, file.roots.synced]
        .into_iter()
        .flatten()
    {
        collect(&root, &root.name, &mut bookmarks);
    }
    Ok(BookmarksFile {
        checksum: file.checksum,
        bookmarks,
    })
}

fn collect(folder: &Node, path: &str, out: &mut Vec<NewBookmark>) {
    for node in &folder.children {
        match (node.kind.as_str(), &node.url) {
            ("url", Some(url)) if is_web(url) => out.push(NewBookmark {
                url: url.clone(),
                title: if node.name.trim().is_empty() {
                    url.clone()
                } else {
                    node.name.clone()
                },
                added_at: unix_seconds(&node.date_added),
                browser_guid: node.guid.clone(),
                browser_folder: Some(path.to_owned()),
            }),
            ("folder", _) => collect(node, &format!("{path}/{}", node.name), out),
            _ => {}
        }
    }
}

/// Internal pages (`chrome://`), bookmarklets and files are not worth organizing.
fn is_web(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

/// Chromium stores microseconds since 1601 as a decimal string.
fn unix_seconds(chromium_time: &str) -> i64 {
    chromium_time.parse::<i64>().map_or(0, |micros| {
        (micros / 1_000_000 - CHROMIUM_EPOCH_OFFSET).max(0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_nested_folders_and_skips_internal_pages() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Bookmarks");
        std::fs::write(
            &path,
            r#"{ "checksum": "abc", "version": 1, "roots": {
                "bookmark_bar": { "type": "folder", "name": "Bookmarks Bar", "children": [
                    { "type": "url", "name": "Rust", "url": "https://rust-lang.org/", "guid": "g1",
                      "date_added": "13407715397118868" },
                    { "type": "url", "name": "Flags", "url": "chrome://flags" },
                    { "type": "folder", "name": "Recipes", "children": [
                        { "type": "url", "name": "", "url": "https://cook.example/pancakes" } ] } ] },
                "other": { "type": "folder", "name": "Other Bookmarks", "children": [] } } }"#,
        )
        .unwrap();

        let file = read_bookmarks(&path).unwrap();
        assert_eq!(file.checksum, "abc");
        let summary: Vec<_> = file
            .bookmarks
            .iter()
            .map(|b| {
                (
                    b.title.as_str(),
                    b.browser_folder.as_deref().unwrap(),
                    b.added_at,
                )
            })
            .collect();
        assert_eq!(
            summary,
            [
                ("Rust", "Bookmarks Bar", 1_763_241_797),
                ("https://cook.example/pancakes", "Bookmarks Bar/Recipes", 0)
            ]
        );
    }
}
