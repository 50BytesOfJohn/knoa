//! The library's two connections (ADR 0015): one that writes, and a read-only one for the
//! UI's reads.
//!
//! In WAL mode the reader neither waits for the writer nor holds it up, so loading the
//! library doesn't stall the organizer's saves, and an import doesn't stall the UI.

use std::{
    ops::Deref,
    path::Path,
    sync::{Mutex, MutexGuard, PoisonError},
};

use knoa_store::Store;

pub struct Database {
    writer: Mutex<Store>,
    reader: Mutex<Store>,
}

/// The read-only connection, locked. It gives out `&Store` only, so writes through it
/// don't compile.
pub struct Reader<'a>(MutexGuard<'a, Store>);

impl Deref for Reader<'_> {
    type Target = Store;

    fn deref(&self) -> &Store {
        &self.0
    }
}

impl Database {
    /// Opens or creates the database, brings its schema up to date, then opens the reader.
    pub fn open(path: &Path) -> knoa_store::Result<Database> {
        let writer = Store::open(path)?;
        let reader = Store::open_read_only(path)?;
        Ok(Database {
            writer: Mutex::new(writer),
            reader: Mutex::new(reader),
        })
    }

    /// For writes, and for reads that a write in the same critical section depends on.
    pub fn write(&self) -> MutexGuard<'_, Store> {
        lock(&self.writer)
    }

    pub fn read(&self) -> Reader<'_> {
        Reader(lock(&self.reader))
    }
}

/// A panic while holding the lock leaves the store usable: SQLite transactions roll back
/// on their own.
fn lock(store: &Mutex<Store>) -> MutexGuard<'_, Store> {
    store.lock().unwrap_or_else(PoisonError::into_inner)
}
