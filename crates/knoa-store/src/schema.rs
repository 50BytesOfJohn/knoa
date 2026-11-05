//! Versioned migrations, tracked in `PRAGMA user_version`.

use rusqlite::Connection;

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
];

/// Runs with foreign keys off: rebuilding a table drops the old one, which would otherwise
/// cascade into every row that refers to it. The pragma can't change inside a transaction.
pub fn migrate(conn: &mut Connection) -> rusqlite::Result<()> {
    conn.pragma_update(None, "foreign_keys", "OFF")?;
    let current: usize = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(current) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", index + 1)?;
        tx.commit()?;
    }
    conn.pragma_update(None, "foreign_keys", "ON")
}

#[cfg(test)]
mod tests {
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
}
