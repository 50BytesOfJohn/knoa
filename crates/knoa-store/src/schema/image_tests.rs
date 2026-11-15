//! Organizing images again now that the AI sees them.

use rusqlite::Connection;

use super::{MIGRATIONS, migrate};
use crate::Store;

/// A database just before this migration, the eleventh, holding `rows`.
fn before_migration(rows: &str) -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    let previous = 10;
    for sql in &MIGRATIONS[..previous] {
        conn.execute_batch(sql).unwrap();
    }
    conn.pragma_update(None, "user_version", previous).unwrap();
    conn.execute_batch(rows).unwrap();
    conn
}

#[test]
fn organized_images_are_queued_once_more() {
    let mut conn = before_migration(
        "INSERT INTO bookmarks (id, workspace_id, url, title, site, added_at, kind, status, deleted_at, fields) VALUES
             (1, 1, 'https://a.dev/a.jpg', 'A', 'a.dev', 1, 'image', 'organized', NULL, '{}'),
             (2, 1, 'https://a.dev/b.jpg', 'B', 'a.dev', 2, 'image', 'organized', 5, '{}'),
             (3, 1, 'https://a.dev/c', 'C', 'a.dev', 3, 'article', 'organized', NULL, '{}'),
             (4, 1, 'https://a.dev/d.jpg', 'D', 'a.dev', 4, 'image', 'failed', NULL, '{}'),
             (5, 1, 'https://a.dev/e.jpg', 'E', 'a.dev', 5, 'image', 'organized', NULL,
                 '{\"medium\":\"photo\"}');
         INSERT INTO jobs (bookmark_id, folders_only) VALUES (1, 1);",
    );

    migrate(&mut conn).unwrap();

    let queued: Vec<(i64, String, bool, bool)> = conn
        .prepare(
            "SELECT b.id, b.status, j.folders_only, j.vision_backfill FROM bookmarks b
             JOIN jobs j ON j.bookmark_id = b.id ORDER BY b.id",
        )
        .unwrap()
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(queued, [(1, "queued".to_owned(), false, true)]);
}

#[test]
fn a_backfill_another_mac_finished_first_is_dropped_instead_of_run() {
    let conn = before_migration(
        "INSERT INTO bookmarks (id, workspace_id, url, title, site, added_at, kind, status,
                                organized_at, fields) VALUES
             (1, 1, 'https://a.dev/a.jpg', 'A', 'a.dev', 1, 'image', 'organized', 1, '{}');",
    );
    let mut store = Store::init(conn).unwrap();
    // A pull brings the first Mac's answer, which looked at the picture.
    store
        .conn
        .execute(
            "UPDATE bookmarks SET fields = '{\"medium\":\"photo\"}' WHERE id = 1",
            [],
        )
        .unwrap();

    assert!(store.take_jobs(i64::MAX, 10).unwrap().is_empty());
    let status: String = store
        .conn
        .query_row("SELECT status FROM bookmarks WHERE id = 1", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(status, "organized");
}
