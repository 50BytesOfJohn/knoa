//! Versioned migrations, tracked in `PRAGMA user_version`.

use rusqlite::Connection;

/// Each entry moves the schema one version forward. Never edit a shipped entry; append.
const MIGRATIONS: &[&str] = &[r"
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
"];

pub fn migrate(conn: &mut Connection) -> rusqlite::Result<()> {
    let current: usize = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(current) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", index + 1)?;
        tx.commit()?;
    }
    Ok(())
}
