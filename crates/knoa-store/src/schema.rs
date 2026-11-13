//! Versioned migrations, tracked in `PRAGMA user_version`.

use rusqlite::Connection;

use crate::{Error, Result};

/// Each entry moves the schema one version forward. Never edit a shipped entry; append.
const MIGRATIONS: &[&str] = &[
    r"
CREATE TABLE bookmarks (
    id                   INTEGER PRIMARY KEY,
    url                  TEXT NOT NULL UNIQUE,
    title                TEXT NOT NULL,
    site                 TEXT NOT NULL,
    added_at             INTEGER NOT NULL,
    kind                 TEXT,
    summary              TEXT,
    fields               TEXT NOT NULL DEFAULT '{}',
    extras               TEXT NOT NULL DEFAULT '[]',
    status               TEXT NOT NULL DEFAULT 'new',
    up_next              INTEGER NOT NULL DEFAULT 0,
    image_path           TEXT,
    image_width          INTEGER,
    image_height         INTEGER,
    page_text            TEXT,
    page_evidence        TEXT,
    fetched_at           INTEGER,
    source               TEXT,
    browser_guid         TEXT,
    browser_folder       TEXT,
    removed_from_browser INTEGER NOT NULL DEFAULT 0,
    model                TEXT,
    error                TEXT,
    organized_at         INTEGER
);
CREATE INDEX bookmarks_added ON bookmarks (added_at DESC);
CREATE INDEX bookmarks_source ON bookmarks (source);

CREATE TABLE folders (
    id          INTEGER PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    auto_assign INTEGER NOT NULL DEFAULT 1,
    color       INTEGER NOT NULL DEFAULT 0,
    position    INTEGER NOT NULL
);

CREATE TABLE bookmark_folders (
    bookmark_id INTEGER NOT NULL REFERENCES bookmarks (id) ON DELETE CASCADE,
    folder_id   INTEGER NOT NULL REFERENCES folders (id) ON DELETE CASCADE,
    origin      TEXT NOT NULL,
    probability REAL,
    PRIMARY KEY (bookmark_id, folder_id)
);

-- Pairs the user removed; the AI never adds them back.
CREATE TABLE folder_exclusions (
    bookmark_id INTEGER NOT NULL REFERENCES bookmarks (id) ON DELETE CASCADE,
    folder_id   INTEGER NOT NULL REFERENCES folders (id) ON DELETE CASCADE,
    PRIMARY KEY (bookmark_id, folder_id)
);

CREATE TABLE bookmark_tags (
    bookmark_id INTEGER NOT NULL REFERENCES bookmarks (id) ON DELETE CASCADE,
    tag         TEXT NOT NULL,
    origin      TEXT NOT NULL,
    PRIMARY KEY (bookmark_id, tag)
);
CREATE INDEX bookmark_tags_tag ON bookmark_tags (tag);

CREATE TABLE tag_exclusions (
    bookmark_id INTEGER NOT NULL REFERENCES bookmarks (id) ON DELETE CASCADE,
    tag         TEXT NOT NULL,
    PRIMARY KEY (bookmark_id, tag)
);

-- The durable organizing queue: a row per bookmark waiting for fetch and AI.
CREATE TABLE jobs (
    bookmark_id INTEGER PRIMARY KEY REFERENCES bookmarks (id) ON DELETE CASCADE,
    attempts    INTEGER NOT NULL DEFAULT 0,
    run_after   INTEGER NOT NULL DEFAULT 0
);

CREATE VIRTUAL TABLE bookmarks_fts USING fts5 (
    title, url, summary, tags, details, body,
    tokenize = 'unicode61 remove_diacritics 2'
);
",
    r"
CREATE TABLE workspaces (
    id             INTEGER PRIMARY KEY,
    name           TEXT NOT NULL,
    color          INTEGER NOT NULL DEFAULT 0,
    position       INTEGER NOT NULL,
    instructions   TEXT NOT NULL DEFAULT '',
    screening      INTEGER NOT NULL DEFAULT 0,
    guideline      TEXT NOT NULL DEFAULT ''
);
INSERT INTO workspaces (id, name, position) VALUES (1, 'Personal', 1);

-- Browser profiles synced into each workspace, as `helium:Default`.
CREATE TABLE workspace_browsers (
    workspace_id INTEGER NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    browser      TEXT NOT NULL,
    PRIMARY KEY (workspace_id, browser)
);

-- URLs are unique per workspace now, so the table is rebuilt (SQLite can't drop a
-- constraint). Ids are kept: the index, folders, tags and jobs refer to them.
CREATE TABLE bookmarks_new (
    id                   INTEGER PRIMARY KEY,
    workspace_id         INTEGER NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    url                  TEXT NOT NULL,
    title                TEXT NOT NULL,
    site                 TEXT NOT NULL,
    added_at             INTEGER NOT NULL,
    kind                 TEXT,
    summary              TEXT,
    fields               TEXT NOT NULL DEFAULT '{}',
    extras               TEXT NOT NULL DEFAULT '[]',
    status               TEXT NOT NULL DEFAULT 'new',
    up_next              INTEGER NOT NULL DEFAULT 0,
    image_path           TEXT,
    image_width          INTEGER,
    image_height         INTEGER,
    page_text            TEXT,
    page_evidence        TEXT,
    fetched_at           INTEGER,
    source               TEXT,
    browser_guid         TEXT,
    browser_folder       TEXT,
    removed_from_browser INTEGER NOT NULL DEFAULT 0,
    model                TEXT,
    error                TEXT,
    organized_at         INTEGER,
    -- Why the AI found it doesn't fit the workspace's guideline; NULL when it fits.
    set_aside            TEXT,
    -- The user kept it (or added it by hand): the guideline never sets it aside.
    kept                 INTEGER NOT NULL DEFAULT 0,
    UNIQUE (workspace_id, url)
);
INSERT INTO bookmarks_new (
    id, workspace_id, url, title, site, added_at, kind, summary, fields, extras, status, up_next,
    image_path, image_width, image_height, page_text, page_evidence, fetched_at, source,
    browser_guid, browser_folder, removed_from_browser, model, error, organized_at
)
SELECT
    id, 1, url, title, site, added_at, kind, summary, fields, extras, status, up_next,
    image_path, image_width, image_height, page_text, page_evidence, fetched_at, source,
    browser_guid, browser_folder, removed_from_browser, model, error, organized_at
FROM bookmarks;
DROP TABLE bookmarks;
ALTER TABLE bookmarks_new RENAME TO bookmarks;
CREATE INDEX bookmarks_added ON bookmarks (workspace_id, added_at DESC);
CREATE INDEX bookmarks_source ON bookmarks (source);

ALTER TABLE folders ADD COLUMN workspace_id INTEGER NOT NULL DEFAULT 1
    REFERENCES workspaces (id) ON DELETE CASCADE;
",
    r"
-- The browser profile bookmarks open in, as `helium:Default`; NULL for the default browser.
ALTER TABLE workspaces ADD COLUMN opens_in TEXT;
",
    r"
-- When the user moved the bookmark to Trash; NULL while it is in the library.
ALTER TABLE bookmarks ADD COLUMN deleted_at INTEGER;

-- Bookmarks deleted for good while their browser still had them, so sync doesn't bring
-- them back. A row goes away once the browser drops the bookmark too.
CREATE TABLE deleted_urls (
    workspace_id INTEGER NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    url          TEXT NOT NULL,
    source       TEXT NOT NULL,
    browser_guid TEXT,
    PRIMARY KEY (workspace_id, url)
);
",
    r"
-- AI usage as daily totals per model, not a log of calls: a few rows a day, whatever the
-- library's size. `day` is the local date, `YYYY-MM-DD`.
CREATE TABLE ai_usage (
    day           TEXT NOT NULL,
    provider      TEXT NOT NULL,
    model         TEXT NOT NULL,
    requests      INTEGER NOT NULL DEFAULT 0,
    input_tokens  INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    cost          REAL NOT NULL DEFAULT 0,
    -- Requests the provider reported no token counts or no cost for.
    unmetered     INTEGER NOT NULL DEFAULT 0,
    unpriced      INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (day, provider, model)
);
",
    r"
-- A folders-only job asks the folder classifier again and keeps everything else the
-- bookmark already has.
ALTER TABLE jobs ADD COLUMN folders_only INTEGER NOT NULL DEFAULT 0;
",
    r"
-- Tags the user wants the AI to use in a workspace, each with an optional hint.
CREATE TABLE workspace_tags (
    workspace_id INTEGER NOT NULL REFERENCES workspaces (id) ON DELETE CASCADE,
    tag          TEXT NOT NULL,
    description  TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (workspace_id, tag)
);
-- The AI may add only listed tags.
ALTER TABLE workspaces ADD COLUMN only_listed_tags INTEGER NOT NULL DEFAULT 0;
",
    r"
-- A hue of the user's own, in degrees; NULL for one of the theme's inks in `color`.
ALTER TABLE folders ADD COLUMN hue INTEGER;
-- A glyph shown instead of the folder shape, by name; NULL for the shape.
ALTER TABLE folders ADD COLUMN icon TEXT;

-- The palette grew from 8 colors to 12 inks, with Pine new at index 3. The old slots
-- become the inks nearest them: those before Pine keep their index, the rest move up one.
-- Old indexes were read modulo 8.
UPDATE folders SET color = CASE WHEN color % 8 < 3 THEN color % 8 ELSE color % 8 + 1 END;
UPDATE workspaces SET color = CASE WHEN color % 8 < 3 THEN color % 8 ELSE color % 8 + 1 END;
",
    r"
-- Cloud sync. The id the server knows a row by, set once the library is linked to an
-- account; NULL in Local mode.
ALTER TABLE workspaces ADD COLUMN sync_id TEXT;
ALTER TABLE folders ADD COLUMN sync_id TEXT;
ALTER TABLE bookmarks ADD COLUMN sync_id TEXT;
CREATE UNIQUE INDEX workspaces_sync_id ON workspaces (sync_id);
CREATE UNIQUE INDEX folders_sync_id ON folders (sync_id);
CREATE UNIQUE INDEX bookmarks_sync_id ON bookmarks (sync_id);

-- The server's thumbnail. `image_path` is this Mac's cached copy.
ALTER TABLE bookmarks ADD COLUMN image_key TEXT;

-- A local image stays pending across pulls until that exact image is uploaded. Every
-- v0.1.5 row with an image needs one, since none has an `image_key` yet; this must run
-- after both columns exist.
ALTER TABLE bookmarks ADD COLUMN pending_image TEXT;
UPDATE bookmarks SET pending_image = hex(randomblob(16))
    WHERE image_path IS NOT NULL AND image_key IS NULL;

-- The title and folder this Mac's browser last gave the bookmark. Imports compare against
-- these, not the synced columns, so two Macs whose browsers disagree don't keep
-- rewriting each other.
ALTER TABLE bookmarks ADD COLUMN seen_title TEXT;
ALTER TABLE bookmarks ADD COLUMN seen_folder TEXT;
UPDATE bookmarks SET seen_title = title, seen_folder = browser_folder WHERE source IS NOT NULL;

-- The account the library is linked to, and how far it has pulled. No row in Local mode.
CREATE TABLE sync_state (
    id      INTEGER PRIMARY KEY CHECK (id = 1),
    user_id TEXT NOT NULL,
    cursor  INTEGER NOT NULL DEFAULT 0
);

-- Local changes waiting to be pushed, oldest first. `data` is the JSON of the columns a
-- `put` sets.
CREATE TABLE outbox (
    seq    INTEGER PRIMARY KEY AUTOINCREMENT,
    mid    TEXT NOT NULL UNIQUE,
    entity TEXT NOT NULL,
    key    TEXT NOT NULL,
    op     TEXT NOT NULL,
    actor  TEXT NOT NULL,
    data   TEXT
);
CREATE INDEX outbox_row ON outbox (entity, key);

-- Pulled changes whose parent hasn't arrived yet, as the change's JSON.
CREATE TABLE sync_inbox (
    entity  TEXT NOT NULL,
    key     TEXT NOT NULL,
    version INTEGER NOT NULL,
    change  TEXT NOT NULL,
    PRIMARY KEY (entity, key)
);

-- Bookmarks the account's plan refused, sent again once it has room.
CREATE TABLE unsent_bookmarks (
    bookmark_id INTEGER PRIMARY KEY REFERENCES bookmarks (id) ON DELETE CASCADE
);
",
    r"
ALTER TABLE bookmarks ADD COLUMN delivery TEXT NOT NULL DEFAULT 'pending_create'
    CHECK (delivery IN ('pending_create', 'blocked_quota', 'blocked_invalid', 'confirmed'));
UPDATE bookmarks SET delivery = CASE
    WHEN id IN (SELECT bookmark_id FROM unsent_bookmarks) THEN 'blocked_quota'
    WHEN EXISTS (SELECT 1 FROM outbox o WHERE o.entity = 'bookmark' AND o.key = bookmarks.sync_id
        AND o.op = 'put' AND json_type(o.data, '$.url') = 'text'
        AND json_type(o.data, '$.workspace_id') = 'text') THEN 'pending_create'
    ELSE 'confirmed' END
    WHERE sync_id IS NOT NULL;
DROP TABLE unsent_bookmarks;
",
];

/// Runs with foreign keys off: rebuilding a table drops the old one, which would otherwise
/// cascade into every row that refers to it. The pragma can't change inside a transaction.
///
/// Refuses a database from a newer Knoa: this version doesn't know its schema and could
/// lose or corrupt what the newer one stored.
pub fn migrate(conn: &mut Connection) -> Result<()> {
    let current: usize = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if current > MIGRATIONS.len() {
        return Err(Error::NewerDatabase {
            found: current,
            supported: MIGRATIONS.len(),
        });
    }
    conn.pragma_update(None, "foreign_keys", "OFF")?;
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(current) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", index + 1)?;
        tx.commit()?;
    }
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use knoa_core::Ink;

    use super::*;

    #[test]
    fn workspace_migration_keeps_bookmarks_and_their_links() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        conn.execute_batch(MIGRATIONS[0]).unwrap();
        conn.pragma_update(None, "user_version", 1).unwrap();
        conn.execute_batch(
            "INSERT INTO bookmarks (id, url, title, site, added_at) VALUES (7, 'https://a.dev/', 'A', 'a.dev', 1);
             INSERT INTO folders (id, name, position) VALUES (3, 'Reading', 1);
             INSERT INTO bookmark_folders VALUES (7, 3, 'user', NULL);
             INSERT INTO bookmark_tags VALUES (7, 'rust', 'ai');
             INSERT INTO jobs (bookmark_id) VALUES (7);",
        )
        .unwrap();

        migrate(&mut conn).unwrap();

        let count = |sql: &str| -> i64 { conn.query_row(sql, [], |row| row.get(0)).unwrap() };
        assert_eq!(count("SELECT workspace_id FROM bookmarks WHERE id = 7"), 1);
        assert_eq!(count("SELECT workspace_id FROM folders WHERE id = 3"), 1);
        assert_eq!(count("SELECT count(*) FROM bookmark_folders"), 1);
        assert_eq!(count("SELECT count(*) FROM bookmark_tags"), 1);
        assert_eq!(count("SELECT count(*) FROM jobs"), 1);
    }

    #[test]
    fn folders_and_workspaces_keep_their_color_in_the_larger_palette() {
        let mut conn = Connection::open_in_memory().unwrap();
        // The schema just before the palette grew.
        let before = 7;
        for sql in &MIGRATIONS[..before] {
            conn.execute_batch(sql).unwrap();
        }
        conn.pragma_update(None, "user_version", before).unwrap();
        // Old slot 6 is Violet now. 12 was read as slot 12 % 8 = 4, which is Sky now.
        conn.execute_batch(
            "UPDATE workspaces SET color = 6 WHERE id = 1;
             INSERT INTO folders (id, name, position, color) VALUES (3, 'Reading', 1, 12);",
        )
        .unwrap();

        migrate(&mut conn).unwrap();

        let color = |sql: &str| -> usize { conn.query_row(sql, [], |row| row.get(0)).unwrap() };
        assert_eq!(
            color("SELECT color FROM workspaces WHERE id = 1"),
            Ink::Violet.index()
        );
        assert_eq!(
            color("SELECT color FROM folders WHERE id = 3"),
            Ink::Sky.index()
        );
    }

    #[test]
    fn the_sync_migration_keeps_the_library_and_leaves_it_local() {
        let mut conn = Connection::open_in_memory().unwrap();
        let before = 8;
        for sql in &MIGRATIONS[..before] {
            conn.execute_batch(sql).unwrap();
        }
        conn.pragma_update(None, "user_version", before).unwrap();
        conn.execute_batch(
            "INSERT INTO folders (id, name, position) VALUES (3, 'Reading', 1);
             INSERT INTO bookmarks (id, workspace_id, url, title, site, added_at, source, browser_folder)
                 VALUES (7, 1, 'https://a.dev/', 'A', 'a.dev', 1, 'helium:Default', 'Bar');
             INSERT INTO bookmarks (id, workspace_id, url, title, site, added_at)
                 VALUES (8, 1, 'https://b.dev/', 'https://b.dev/', 'b.dev', 2);
             UPDATE bookmarks SET image_path = 'a.png' WHERE id = 7;
             INSERT INTO bookmark_folders VALUES (7, 3, 'user', NULL);
             INSERT INTO bookmark_tags VALUES (7, 'rust', 'ai');",
        )
        .unwrap();

        migrate(&mut conn).unwrap();

        let text =
            |sql: &str| -> Option<String> { conn.query_row(sql, [], |row| row.get(0)).unwrap() };
        assert_eq!(
            text("SELECT seen_title FROM bookmarks WHERE id = 7").as_deref(),
            Some("A")
        );
        assert_eq!(
            text("SELECT seen_folder FROM bookmarks WHERE id = 7").as_deref(),
            Some("Bar")
        );
        assert_eq!(
            text("SELECT seen_title FROM bookmarks WHERE id = 8"),
            None,
            "added by hand"
        );
        let count = |sql: &str| -> i64 { conn.query_row(sql, [], |row| row.get(0)).unwrap() };
        assert_eq!(text("SELECT max(sync_id) FROM bookmarks"), None);
        assert_eq!(
            count("SELECT length(pending_image) FROM bookmarks WHERE id = 7"),
            32,
            "a v0.1.5 image still has to be uploaded"
        );
        assert_eq!(
            text("SELECT pending_image FROM bookmarks WHERE id = 8"),
            None
        );
        assert_eq!(
            count("SELECT count(*) FROM bookmarks WHERE delivery = 'pending_create'"),
            2
        );
        assert_eq!(count("SELECT count(*) FROM bookmark_folders"), 1);
        assert_eq!(count("SELECT count(*) FROM bookmark_tags"), 1);
        assert_eq!(count("SELECT count(*) FROM sync_state"), 0);
    }

    #[test]
    fn a_database_from_a_newer_knoa_is_refused() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "user_version", MIGRATIONS.len() + 1)
            .unwrap();
        assert!(matches!(
            migrate(&mut conn),
            Err(Error::NewerDatabase { .. })
        ));
    }
}

#[cfg(test)]
mod delivery_tests;
