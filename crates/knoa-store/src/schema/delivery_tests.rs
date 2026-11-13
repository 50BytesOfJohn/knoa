//! Upgrading publication evidence from the schema before explicit delivery states.

use super::MIGRATIONS;
use crate::Store;
use rusqlite::Connection;

#[test]
fn delivery_upgrade_preserves_confirmed_pending_and_parked_bookmarks() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("previous.db");
    {
        let conn = Connection::open(&path).unwrap();
        let previous = MIGRATIONS.len() - 1;
        for sql in &MIGRATIONS[..previous] {
            conn.execute_batch(sql).unwrap();
        }
        conn.pragma_update(None, "user_version", previous).unwrap();
        conn.execute_batch(
            r#"
INSERT INTO sync_state (id, user_id) VALUES (1, 'upgrade-fixture');
UPDATE workspaces SET sync_id = 'workspace' WHERE id = 1;
INSERT INTO bookmarks (id, workspace_id, url, title, site, added_at, sync_id) VALUES
    (1, 1, 'https://confirmed.dev/', 'Confirmed', 'confirmed.dev', 1, 'confirmed'),
    (2, 1, 'https://edited.dev/', 'Edited', 'edited.dev', 2, 'edited'),
    (3, 1, 'https://pending.dev/', 'Pending', 'pending.dev', 3, 'pending'),
    (4, 1, 'https://parked.dev/', 'Parked', 'parked.dev', 4, 'parked'),
    (5, 1, 'https://local.dev/', 'Local', 'local.dev', 5, NULL);
INSERT INTO outbox (mid, entity, key, op, actor, data) VALUES
    ('edit', 'bookmark', 'edited', 'put', 'user', '{"title":"Edited"}'),
    ('create', 'bookmark', 'pending', 'put', 'user',
        '{"workspace_id":"workspace","url":"https://pending.dev/","title":"Pending"}');
INSERT INTO unsent_bookmarks VALUES (4);
"#,
        )
        .unwrap();
    }

    let store = Store::open(&path).unwrap();
    let states: Vec<(String, String)> = store
        .conn
        .prepare("SELECT title, delivery FROM bookmarks ORDER BY id")
        .unwrap()
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(
        states,
        [
            ("Confirmed".into(), "confirmed".into()),
            ("Edited".into(), "confirmed".into()),
            ("Pending".into(), "pending_create".into()),
            ("Parked".into(), "blocked_quota".into()),
            ("Local".into(), "pending_create".into()),
        ],
        "a pending edit must not hide an already published bookmark"
    );
    assert_eq!(store.unsent_bookmarks().unwrap(), 1);
    assert_eq!(
        store.next_push().unwrap().len(),
        2,
        "upgrade keeps the outbox"
    );
    drop(store);
    let mut reopened = Store::open(&path).unwrap();
    assert_eq!(reopened.unsent_bookmarks().unwrap(), 1);
    assert_eq!(reopened.resend_unsent().unwrap(), 1);
    assert_eq!(reopened.next_push().unwrap().len(), 3);
}
