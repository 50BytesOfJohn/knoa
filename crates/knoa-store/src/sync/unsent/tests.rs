//! Creation retries and retention of local changes across parking.

use knoa_core::{Image, Origin, Page, Place, Query, protocol::bookmark_id};
use serde_json::{Value, json};

use super::*;
use crate::{
    NewBookmarks,
    test_support::{
        WORKSPACE, WORKSPACE_KEY, applied, bookmark, bookmark_row, change, import_one, linked,
        pushed,
    },
};

fn answer(mutation: &Value, reason: &str) -> MutationResult {
    serde_json::from_value(json!({
        "mid": mutation["mid"], "status": "rejected", "reason": reason,
    }))
    .unwrap()
}

#[test]
fn a_pulled_invalid_creation_accepts_the_server_row_without_retrying() {
    let mut store = Store::open_in_memory().unwrap();
    linked(&mut store);
    let url = "https://a.dev/";
    store
        .import(
            WORKSPACE,
            "helium:Default",
            &[bookmark(url, &"x".repeat(2001), 1)],
            NewBookmarks::Organize,
        )
        .unwrap();
    let id = store.search(&Query::default(), 1).unwrap()[0].id;
    let put = pushed(&store).remove(0);
    store.settle(&[answer(&put, "invalid")]).unwrap();
    let key = bookmark_id(WORKSPACE_KEY, url).unwrap();

    store
        .apply_remote(
            &[change(json!({
                "entity": "bookmark", "key": key, "version": 2, "deleted": false,
                "data": bookmark_row(&key, WORKSPACE_KEY, url, "Remote title"),
            }))],
            2,
        )
        .unwrap();

    assert_eq!(store.bookmark(id).unwrap().unwrap().title, "Remote title");
    assert_eq!(pushed(&store), Vec::<Value>::new());
    assert_eq!(store.cloud_jobs(10).unwrap().len(), 1);
}

#[test]
fn releasing_blocked_cloud_jobs_restores_status_and_preserves_pending_jobs() {
    use knoa_core::{Enrichment, Fields, Kind, Status};

    let mut store = Store::open_in_memory().unwrap();
    linked(&mut store);
    let quota = import_one(&mut store);
    store
        .finish_organized(
            quota,
            &Enrichment {
                kind: Kind::Article,
                summary: None,
                fields: Fields::new(),
                extras: Vec::new(),
                tags: Vec::new(),
                folders: Vec::new(),
                set_aside: None,
                model: "test".into(),
            },
            5,
        )
        .unwrap();
    store.queue(quota).unwrap();
    let put = pushed(&store).remove(0);
    store.settle(&[answer(&put, "quota")]).unwrap();
    let invalid = store
        .add_link(WORKSPACE, "https://invalid.dev/", 10)
        .unwrap()
        .id;
    let put = pushed(&store).remove(0);
    store.settle(&[answer(&put, "invalid")]).unwrap();
    let pending = store
        .add_link(WORKSPACE, "https://pending.dev/", 10)
        .unwrap()
        .id;
    let outbox = pushed(&store);

    assert_eq!(store.cloud_jobs(10).unwrap(), []);
    for id in [quota, invalid] {
        assert_eq!(store.bookmark(id).unwrap().unwrap().status, Status::Queued);
    }
    store.release_blocked_jobs().unwrap();

    assert_eq!(
        store.bookmark(quota).unwrap().unwrap().status,
        Status::Organized
    );
    assert_eq!(
        store.bookmark(invalid).unwrap().unwrap().status,
        Status::New
    );
    assert_eq!(
        store.bookmark(pending).unwrap().unwrap().status,
        Status::Queued
    );
    assert_eq!(pushed(&store), outbox);
    let jobs = store.take_jobs(i64::MAX, 10).unwrap();
    assert_eq!(jobs.len(), 1, "blocked bookmarks must leave the queue");
    assert_eq!(jobs[0].bookmark.id, pending);
}

#[test]
fn a_pulled_parked_bookmark_retains_local_edits_and_resumes_syncing() {
    let mut store = Store::open_in_memory().unwrap();
    linked(&mut store);
    let id = import_one(&mut store);
    let put = pushed(&store).remove(0);
    store.settle(&[answer(&put, "quota")]).unwrap();
    store.set_up_next(id, true).unwrap();
    store.add_tag(id, "rust", Origin::User).unwrap();
    let page = Page {
        text: "Local page text".into(),
        description: Some("Local summary".into()),
        ..Page::default()
    };
    store.save_page(id, &page, 10).unwrap();
    assert_eq!(pushed(&store), Vec::<Value>::new());
    let key = bookmark_id(WORKSPACE_KEY, "https://a.dev/").unwrap();
    let changes = [
        change(json!({
            "entity": "bookmark", "key": key, "version": 2, "deleted": false,
            "data": bookmark_row(&key, WORKSPACE_KEY, "https://a.dev/", "Remote title"),
        })),
        change(json!({
            "entity": "bookmark_tag", "key": format!("{key}/rust"),
            "version": 3, "deleted": false,
            "data": { "bookmark_id": key, "tag": "rust", "origin": "user", "excluded": true },
        })),
        change(json!({
            "entity": "bookmark_text", "key": key, "version": 4, "deleted": false,
            "data": { "bookmark_id": key, "text": "Remote text" },
        })),
    ];

    store.apply_remote(&changes, 4).unwrap();

    assert_eq!(
        store.cloud_jobs(10).unwrap().len(),
        1,
        "a pulled live row confirms delivery"
    );
    let saved = store.bookmark(id).unwrap().unwrap();
    assert_eq!(saved.title, "A");
    assert!(saved.up_next);
    assert_eq!(saved.summary, page.description);
    assert_eq!(saved.tags, [("rust".into(), Origin::User)]);
    let found = store
        .search(&Query::parse("local page text", None, Place::Everything), 1)
        .unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, id);
    assert_eq!(store.unsent_bookmarks().unwrap(), 0);
    let pending = pushed(&store);
    assert_eq!(pending.len(), 3);
    assert_eq!(pending[0]["data"]["title"], "A");
    assert_eq!(pending[0]["data"]["up_next"], true);
    store.settle(&applied(&store)).unwrap();
    store.set_up_next(id, false).unwrap();
    let [edit] = <[Value; 1]>::try_from(pushed(&store)).unwrap();
    assert_eq!(edit["data"], json!({ "up_next": false }));
}

#[test]
fn correcting_an_invalid_creation_retries_the_full_bookmark_before_handoff() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.db");
    let mut store = Store::open(&path).unwrap();
    linked(&mut store);
    let title = "x".repeat(2001);
    store
        .import(
            WORKSPACE,
            "helium:Default",
            &[bookmark("https://a.dev/", &title, 1)],
            NewBookmarks::Organize,
        )
        .unwrap();
    let id = store.search(&Query::default(), 1).unwrap()[0].id;
    store.add_tag(id, "rust", Origin::User).unwrap();
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
    let saved = store.bookmark(id).unwrap().unwrap();
    let identity = store.bookmark_identity(&saved).unwrap().unwrap();
    store
        .set_image_if_current(
            &identity,
            &Image {
                path: "/cache/a.png".into(),
                width: 10,
                height: 10,
            },
        )
        .unwrap();
    let put = pushed(&store).remove(0);

    assert_eq!(
        store.settle(&[answer(&put, "invalid")]).unwrap(),
        Settled {
            parked: 0,
            invalid: 1
        }
    );
    drop(store);
    let mut store = Store::open(&path).unwrap();

    assert!(
        pushed(&store).is_empty(),
        "children wait with the refused creation"
    );
    assert_eq!(store.cloud_jobs(10).unwrap(), []);
    assert_eq!(store.thumbnail_uploads(10).unwrap(), []);
    store
        .import(
            WORKSPACE,
            "helium:Default",
            &[bookmark("https://a.dev/", &title, 1)],
            NewBookmarks::Organize,
        )
        .unwrap();
    assert!(
        pushed(&store).is_empty(),
        "an unchanged import must not retry"
    );
    store
        .import(
            WORKSPACE,
            "helium:Default",
            &[bookmark("https://a.dev/", "Corrected", 1)],
            NewBookmarks::Organize,
        )
        .unwrap();

    let pending = pushed(&store);
    assert_eq!(pending.len(), 3, "row, text and tag retry together");
    assert_eq!(pending[0]["data"]["title"], "Corrected");
    assert_eq!(pending[0]["data"]["url"], "https://a.dev/");
    assert_eq!(pending[0]["data"]["workspace_id"], WORKSPACE_KEY);
    assert_eq!(pending[1]["data"]["text"], "Full page text");
    assert_eq!(pending[2]["data"]["tag"], "rust");
    assert_eq!(store.cloud_jobs(10).unwrap(), []);
    assert_eq!(store.thumbnail_uploads(10).unwrap(), []);
    store
        .settle(&[
            serde_json::from_value(json!({"mid": pending[0]["mid"], "status": "applied"})).unwrap(),
        ])
        .unwrap();
    assert_eq!(store.cloud_jobs(10).unwrap().len(), 1);
    assert_eq!(store.thumbnail_uploads(10).unwrap().len(), 1);
}

#[test]
fn a_bookmark_past_the_plans_limit_goes_again_whole_when_resent() {
    let mut store = Store::open_in_memory().unwrap();
    linked(&mut store);
    store
        .import(
            WORKSPACE,
            "helium:Default",
            &[bookmark("https://a.dev/", "A", 1)],
            NewBookmarks::Organize,
        )
        .unwrap();
    let id = store.search(&Query::default(), 1).unwrap()[0].id;
    let saved = store.bookmark(id).unwrap().unwrap();
    let identity = store.bookmark_identity(&saved).unwrap().unwrap();
    store
        .set_image_if_current(
            &identity,
            &Image {
                path: "/cache/a.png".into(),
                width: 10,
                height: 10,
            },
        )
        .unwrap();
    assert_eq!(store.cloud_jobs(10).unwrap(), []);
    assert_eq!(store.thumbnail_uploads(10).unwrap(), []);
    store.add_tag(id, "rust", Origin::User).unwrap();
    let [put, tag] = <[Value; 2]>::try_from(pushed(&store)).unwrap();

    let settled = store
        .settle(&[answer(&put, "quota"), answer(&tag, "invalid")])
        .unwrap();
    store.add_tag(id, "web", Origin::User).unwrap();

    assert_eq!(
        settled,
        Settled {
            parked: 1,
            invalid: 0
        }
    );
    assert_eq!(pushed(&store), Vec::<Value>::new());
    assert_eq!(store.unsent_bookmarks().unwrap(), 1);

    assert_eq!(
        store.cloud_jobs(10).unwrap(),
        [],
        "quota parking must not publish the row"
    );
    assert_eq!(store.thumbnail_uploads(10).unwrap(), []);
    assert_eq!(store.resend_unsent().unwrap(), 1);
    let pending = pushed(&store);
    assert_eq!(pending[0]["data"]["url"], "https://a.dev/");
    assert_eq!(pending[0]["data"]["title"], "A");
    assert_eq!(pending[0]["data"]["workspace_id"], WORKSPACE_KEY);
    assert_eq!(store.cloud_jobs(10).unwrap(), []);
    assert_eq!(store.thumbnail_uploads(10).unwrap(), []);
    let entities: Vec<Value> = pushed(&store).iter().map(|m| m["entity"].clone()).collect();
    assert_eq!(
        entities,
        [
            json!("bookmark"),
            json!("bookmark_tag"),
            json!("bookmark_tag")
        ]
    );
    assert_eq!(store.unsent_bookmarks().unwrap(), 0);
    store.settle(&applied(&store)).unwrap();
    assert_eq!(store.cloud_jobs(10).unwrap().len(), 1);
    assert_eq!(store.thumbnail_uploads(10).unwrap().len(), 1);
}
