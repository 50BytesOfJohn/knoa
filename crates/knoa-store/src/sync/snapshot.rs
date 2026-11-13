//! Recording a bookmark's full synchronized state, including child ownership and exclusions.

use knoa_core::Origin;
use rusqlite::{Connection, params_from_iter};

use super::Recorder;
use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Bookmarks {
    All,
    One(i64),
}

pub(super) fn record_bookmark_snapshot(conn: &Connection, bookmark: i64) -> Result<()> {
    record_bookmark_content(conn, bookmark)?;
    record_bookmark_links(conn, Bookmarks::One(bookmark))
}

/// Linking exports links for adopted bookmarks too, without replacing their content.
pub(super) fn record_bookmark_content(conn: &Connection, bookmark: i64) -> Result<()> {
    Recorder::user(conn)?.put_whole_bookmark(bookmark)?;
    Recorder::ai(conn)?.text(bookmark)
}

pub(super) fn record_bookmark_links(conn: &Connection, bookmarks: Bookmarks) -> Result<()> {
    let user = Recorder::user(conn)?;
    let ai = Recorder::ai(conn)?;
    let by = |origin: &str| {
        if origin == Origin::Ai.slug() {
            &ai
        } else {
            &user
        }
    };
    let bookmark = match bookmarks {
        Bookmarks::All => None,
        Bookmarks::One(id) => Some(id),
    };
    let filter = if bookmark.is_some() {
        "WHERE bookmark_id = ?1"
    } else {
        ""
    };
    let tags = rows::<(i64, String, String)>(
        conn,
        &format!(
            "SELECT bookmark_id, tag, origin FROM bookmark_tags {filter}
         UNION ALL SELECT bookmark_id, tag, 'user' FROM tag_exclusions {filter}"
        ),
        bookmark,
    )?;
    for (bookmark, tag, origin) in tags {
        by(&origin).tag(bookmark, &tag)?;
    }
    let links = rows::<(i64, i64, String)>(
        conn,
        &format!(
            "SELECT bookmark_id, folder_id, origin FROM bookmark_folders {filter}
         UNION ALL SELECT bookmark_id, folder_id, 'user' FROM folder_exclusions {filter}"
        ),
        bookmark,
    )?;
    for (bookmark, folder, origin) in links {
        by(&origin).folder_link(bookmark, folder)?;
    }
    Ok(())
}

fn rows<T>(conn: &Connection, sql: &str, bookmark: Option<i64>) -> Result<Vec<T>>
where
    T: for<'r> TryFrom<&'r rusqlite::Row<'r>, Error = rusqlite::Error>,
{
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params_from_iter(bookmark), |row| T::try_from(row))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

#[cfg(test)]
mod tests {
    use knoa_core::{Enrichment, Fields, FolderColor, Kind, Page};
    use serde_json::{Value, json};

    use super::*;
    use crate::{
        Store,
        test_support::{WORKSPACE, import_one, linked, mids, pushed},
    };

    #[test]
    fn linking_and_resending_export_the_same_bookmark_state() {
        let mut store = Store::open_in_memory().unwrap();
        let id = import_one(&mut store);
        let folders: Vec<_> = ["AI", "User", "Excluded"]
            .map(|name| {
                store
                    .create_folder(WORKSPACE, name, "", true, FolderColor::default(), None)
                    .unwrap()
            })
            .into();
        store
            .save_page(
                id,
                &Page {
                    text: "Full page text".into(),
                    ..Page::default()
                },
                5,
            )
            .unwrap();
        store
            .finish_organized(
                id,
                &Enrichment {
                    kind: Kind::Article,
                    summary: None,
                    fields: Fields::new(),
                    extras: Vec::new(),
                    tags: vec!["ai".into()],
                    folders: vec![(folders[0], 0.9)],
                    set_aside: None,
                    model: "test".into(),
                },
                10,
            )
            .unwrap();
        store.add_tag(id, "user", Origin::User).unwrap();
        store.remove_tag(id, "excluded").unwrap();
        store.add_to_folder(id, folders[1]).unwrap();
        store.remove_from_folder(id, folders[2]).unwrap();
        linked(&mut store);
        let initial = bookmark_mutations(&store);
        let data: Vec<_> = initial
            .iter()
            .map(|mutation| (&mutation["entity"], &mutation["data"], &mutation["actor"]))
            .collect();
        assert_eq!(
            initial.len(),
            8,
            "row, text, three tags and three folder links: {data:?}"
        );
        for entity in ["bookmark_tag", "bookmark_folder"] {
            let children: Vec<_> = initial
                .iter()
                .filter(|mutation| mutation["entity"] == entity)
                .collect();
            assert!(children.iter().any(|mutation| mutation["actor"] == "ai" && mutation["data"]["origin"] == "ai"), "{entity} keeps AI ownership");
            assert!(
                children
                    .iter()
                    .any(|mutation| mutation["actor"] == "user"
                        && mutation["data"]["excluded"] == true),
                "{entity} keeps user exclusions"
            );
        }
        assert!(
            initial
                .iter()
                .any(|mutation| mutation["entity"] == "bookmark_text"
                    && mutation["data"]["text"] == "Full page text")
        );
        let put = pushed(&store)
            .into_iter()
            .find(|mutation| mutation["entity"] == "bookmark")
            .unwrap();
        store
            .settle(&[serde_json::from_value(json!({
                "mid": put["mid"], "status": "rejected", "reason": "quota",
            }))
            .unwrap()])
            .unwrap();
        store.acknowledge(&mids(&store)).unwrap();

        assert_eq!(store.resend_unsent().unwrap(), 1);

        assert_eq!(bookmark_mutations(&store), initial);
    }

    #[test]
    fn linking_an_adopted_bookmark_exports_links_without_replacing_its_row_or_text() {
        use crate::{
            AccountRows,
            test_support::{WORKSPACE_KEY, bookmark_row, change, workspace_row},
        };
        use knoa_core::protocol::bookmark_id;

        let mut store = Store::open_in_memory().unwrap();
        let id = import_one(&mut store);
        store.add_tag(id, "user", Origin::User).unwrap();
        store
            .save_page(
                id,
                &Page {
                    text: "Local page text".into(),
                    ..Page::default()
                },
                5,
            )
            .unwrap();
        let url = store.bookmark(id).unwrap().unwrap().url;
        let key = bookmark_id(WORKSPACE_KEY, &url).unwrap();
        let mut account = AccountRows::default();
        account.add(&[
            change(json!({ "entity": "workspace", "key": WORKSPACE_KEY,
                "version": 1, "deleted": false, "data": workspace_row(WORKSPACE_KEY, "Personal") })),
            change(json!({ "entity": "bookmark", "key": key, "version": 2, "deleted": false,
                "data": bookmark_row(&key, WORKSPACE_KEY, &url, "Remote title") })),
        ]);

        store.link("user-fixture", account).unwrap();

        let mutations = bookmark_mutations(&store);
        assert_eq!(mutations.len(), 1);
        assert_eq!(mutations[0]["entity"], "bookmark_tag");
        assert_eq!(mutations[0]["actor"], "user");
    }

    fn bookmark_mutations(store: &Store) -> Vec<Value> {
        pushed(store)
            .into_iter()
            .filter(|mutation| mutation["entity"].as_str().unwrap().starts_with("bookmark"))
            .map(|mut mutation| {
                mutation.as_object_mut().unwrap().remove("mid");
                mutation
            })
            .collect()
    }
}
