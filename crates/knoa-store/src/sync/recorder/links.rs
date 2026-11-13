//! Recording a bookmark's tags, folder links and page text, which are sent whole.

use std::collections::BTreeMap;

use knoa_core::{
    Origin,
    protocol::{BookmarkFolder, BookmarkTag, BookmarkText, Entity, folder_link_key, tag_key},
};

use super::{
    super::{
        outbox,
        rows::{self, Link},
    },
    Recorder,
};
use crate::Result;

impl Recorder<'_> {
    /// The bookmark's tags, to compare with after a change that may touch several.
    pub fn tags_before(&self, bookmark: i64) -> Result<Option<BTreeMap<String, Origin>>> {
        if !self.linked {
            return Ok(None);
        }
        rows::tags(self.conn, bookmark).map(Some)
    }

    /// Records each tag that was added, removed or changed origin since `before`.
    pub fn tags_changed(
        &self,
        bookmark: i64,
        before: Option<BTreeMap<String, Origin>>,
    ) -> Result<()> {
        let Some(before) = before else {
            return Ok(());
        };
        let after = rows::tags(self.conn, bookmark)?;
        let touched = before.keys().chain(after.keys());
        for tag in touched.filter(|tag| before.get(*tag) != after.get(*tag)) {
            self.tag(bookmark, tag)?;
        }
        Ok(())
    }

    /// Records the tag as the bookmark has it now: on it, removed by the user, or gone.
    pub fn tag(&self, bookmark: i64, tag: &str) -> Result<()> {
        if !self.linked {
            return Ok(());
        }
        let Some(bookmark_key) = self.bookmark_key(bookmark)? else {
            return Ok(());
        };
        let key = tag_key(&bookmark_key, tag);
        let (origin, excluded) = match rows::tag(self.conn, bookmark, tag)? {
            Link::On { origin, .. } => (origin, false),
            Link::Excluded => (Origin::User, true),
            Link::Gone => return outbox::delete(self.conn, Entity::BookmarkTag, &key, self.actor),
        };
        let row = BookmarkTag {
            bookmark_id: bookmark_key,
            tag: tag.to_owned(),
            origin,
            excluded,
        };
        self.put(Entity::BookmarkTag, &key, &row)
    }

    /// Records the folder link as the bookmark has it now, like [`Recorder::tag`].
    pub fn folder_link(&self, bookmark: i64, folder: i64) -> Result<()> {
        if !self.linked {
            return Ok(());
        }
        let (Some(bookmark_key), Some(folder_key)) =
            (self.bookmark_key(bookmark)?, self.folder_key(folder)?)
        else {
            return Ok(());
        };
        let key = folder_link_key(&bookmark_key, &folder_key);
        let (origin, probability, excluded) = match rows::folder_link(self.conn, bookmark, folder)?
        {
            Link::On {
                origin,
                probability,
            } => (origin, probability, false),
            Link::Excluded => (Origin::User, None, true),
            Link::Gone => {
                return outbox::delete(self.conn, Entity::BookmarkFolder, &key, self.actor);
            }
        };
        let row = BookmarkFolder {
            bookmark_id: bookmark_key,
            folder_id: folder_key,
            origin,
            probability,
            excluded,
        };
        self.put(Entity::BookmarkFolder, &key, &row)
    }

    /// Records the bookmark's page text, if it has any.
    pub fn text(&self, bookmark: i64) -> Result<()> {
        if !self.linked {
            return Ok(());
        }
        let (Some(key), Some(text)) = (
            self.bookmark_key(bookmark)?,
            rows::page_text(self.conn, bookmark)?,
        ) else {
            return Ok(());
        };
        let row = BookmarkText {
            bookmark_id: key.clone(),
            text,
        };
        self.put(Entity::BookmarkText, &key, &row)
    }
}

#[cfg(test)]
mod tests {
    use knoa_core::{Enrichment, Kind};
    use serde_json::{Value, json};

    use super::*;
    use crate::{
        Store,
        test_support::{applied, import_one, linked, pushed},
    };

    fn organized(tags: &[&str]) -> Enrichment {
        Enrichment {
            kind: Kind::Article,
            summary: Some("About Rust".into()),
            fields: knoa_core::Fields::new(),
            extras: Vec::new(),
            tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
            folders: Vec::new(),
            set_aside: None,
            model: "test".into(),
        }
    }

    #[test]
    fn a_rerun_sends_only_the_tags_that_changed() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let id = import_one(&mut store);
        store.take_jobs(0, 1).unwrap();
        store
            .finish_organized(id, &organized(&["rust", "web"]), 10)
            .unwrap();
        store.settle(&applied(&store)).unwrap();

        store.queue(id).unwrap();
        store.take_jobs(0, 1).unwrap();
        store
            .finish_organized(id, &organized(&["web", "cli"]), 20)
            .unwrap();

        let tags: Vec<(String, String, Value)> = pushed(&store)
            .into_iter()
            .filter(|m| m["entity"] == "bookmark_tag")
            .map(|m| {
                (
                    m["key"]
                        .as_str()
                        .unwrap()
                        .rsplit('/')
                        .next()
                        .unwrap()
                        .to_owned(),
                    m["op"].as_str().unwrap().to_owned(),
                    m["actor"].clone(),
                )
            })
            .collect();
        assert_eq!(
            tags,
            [
                ("rust".to_owned(), "delete".to_owned(), json!("ai")),
                ("cli".to_owned(), "put".to_owned(), json!("ai")),
            ]
        );
    }

    #[test]
    fn removing_a_tag_sends_it_excluded() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let id = import_one(&mut store);
        store.add_tag(id, "rust", Origin::Ai).unwrap();
        store.settle(&applied(&store)).unwrap();

        store.remove_tag(id, "rust").unwrap();

        let [put] = <[Value; 1]>::try_from(pushed(&store)).unwrap();
        assert_eq!(put["actor"], json!("user"));
        assert_eq!(put["data"]["excluded"], json!(true));
    }
}
