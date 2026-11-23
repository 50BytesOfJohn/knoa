//! The title and description the user writes for a bookmark.

use knoa_core::BookmarkId;
use rusqlite::params;

use crate::{Result, Store, search, sync::Recorder};

impl Store {
    /// An empty title is refused by the caller; the URL stands in only until a title exists.
    pub fn rename_bookmark(&mut self, id: BookmarkId, title: &str) -> Result<()> {
        self.write_text(
            id,
            "UPDATE bookmarks SET title = ?2 WHERE id = ?1",
            Some(title),
        )
    }

    /// The user's description, or none: either way the organizer leaves it alone from now on.
    pub fn describe_bookmark(&mut self, id: BookmarkId, summary: Option<&str>) -> Result<()> {
        self.write_text(
            id,
            "UPDATE bookmarks SET summary = ?2, summary_origin = 'user' WHERE id = ?1",
            summary,
        )
    }

    fn write_text(&mut self, id: BookmarkId, sql: &str, text: Option<&str>) -> Result<()> {
        let tx = self.write()?;
        Recorder::user(&tx)?.on_bookmark(id.0, || {
            tx.execute(sql, params![id.0, text])?;
            Ok(())
        })?;
        search::reindex(&tx, id.0)?;
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use knoa_core::{Enrichment, Fields, Kind, Origin};

    use crate::{
        Store,
        test_support::{applied, import_one, linked, pushed},
    };

    #[test]
    fn organizing_again_keeps_the_users_description() {
        let mut store = Store::open_in_memory().unwrap();
        let id = import_one(&mut store);
        store.take_jobs(0, 1).unwrap();
        store.describe_bookmark(id, Some("Mine")).unwrap();

        store.finish_organized(id, &answer("The AI's"), 10).unwrap();

        let saved = store.bookmark(id).unwrap().unwrap();
        assert_eq!(saved.summary.as_deref(), Some("Mine"));
        assert_eq!(saved.summary_origin, Origin::User);
    }

    #[test]
    fn a_linked_description_pushes_who_wrote_it() {
        let mut store = Store::open_in_memory().unwrap();
        linked(&mut store);
        let id = import_one(&mut store);
        store.settle(&applied(&store)).unwrap();

        store.describe_bookmark(id, Some("Mine")).unwrap();

        let push = pushed(&store);
        assert_eq!(push.len(), 1);
        assert_eq!(push[0]["actor"], "user");
        assert_eq!(
            push[0]["data"],
            serde_json::json!({ "summary": "Mine", "summary_origin": "user" })
        );
    }

    fn answer(summary: &str) -> Enrichment {
        Enrichment {
            kind: Kind::Article,
            title: None,
            summary: Some(summary.into()),
            fields: Fields::new(),
            extras: Vec::new(),
            tags: Vec::new(),
            folders: Vec::new(),
            set_aside: None,
            model: "test".into(),
        }
    }
}
