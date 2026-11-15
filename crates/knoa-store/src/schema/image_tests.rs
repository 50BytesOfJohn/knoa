//! Organizing images again now that the AI sees them.

use rusqlite::Connection;

use super::{MIGRATIONS, migrate};

#[test]
fn organized_images_are_queued_once_more() {
    let mut conn = Connection::open_in_memory().unwrap();
    // The schema just before this migration, the eleventh.
    let previous = 10;
    for sql in &MIGRATIONS[..previous] {
        conn.execute_batch(sql).unwrap();
    }
    conn.pragma_update(None, "user_version", previous).unwrap();
    conn.execute_batch(
        "INSERT INTO bookmarks (id, workspace_id, url, title, site, added_at, kind, status, deleted_at) VALUES
             (1, 1, 'https://a.dev/a.jpg', 'A', 'a.dev', 1, 'image', 'organized', NULL),
             (2, 1, 'https://a.dev/b.jpg', 'B', 'a.dev', 2, 'image', 'organized', 5),
             (3, 1, 'https://a.dev/c', 'C', 'a.dev', 3, 'article', 'organized', NULL),
             (4, 1, 'https://a.dev/d.jpg', 'D', 'a.dev', 4, 'image', 'failed', NULL);
         INSERT INTO jobs (bookmark_id, folders_only) VALUES (1, 1);",
    )
    .unwrap();

    migrate(&mut conn).unwrap();

    let queued: Vec<(i64, String, bool)> = conn
        .prepare(
            "SELECT b.id, b.status, j.folders_only FROM bookmarks b
             JOIN jobs j ON j.bookmark_id = b.id ORDER BY b.id",
        )
        .unwrap()
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(queued, [(1, "queued".to_owned(), false)]);
}
