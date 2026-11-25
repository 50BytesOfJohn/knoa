//! Counts the test process's heap, so a test can assert the most a piece of work held at
//! once, its decoders' helper threads included. Other tests would add to the count, so a
//! measuring test runs again alone in a process of its own.

#![expect(
    unsafe_code,
    reason = "a global allocator is unsafe to write; this one exists only in test builds"
)]

use std::{
    alloc::{GlobalAlloc, Layout, System},
    env,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

const ALONE: &str = "KNOA_HEAP_ALONE";

/// Whether the calling test should measure: it is the only test in this process. When it
/// isn't, the test reruns alone in a child process, and fails if that run fails.
/// `test` is the test's path within the crate, as `cargo test` names it.
pub fn alone(test: &str) -> bool {
    if env::var_os(ALONE).is_some() {
        return true;
    }
    let exe = env::current_exe().expect("the test binary's path");
    let output = Command::new(exe)
        .args([test, "--exact", "--test-threads=1"])
        .env(ALONE, "1")
        .output()
        .expect("the test binary runs");
    assert!(
        output.status.success(),
        "{test} failed when run alone:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    false
}

/// The most `work` held on the heap at once, in bytes, beyond what was live before it, and
/// its result.
pub fn peak<T>(work: impl FnOnce() -> T) -> (usize, T) {
    let before = LIVE.load(Ordering::Relaxed);
    HIGH.store(before, Ordering::Relaxed);
    let result = work();
    (HIGH.load(Ordering::Relaxed) - before, result)
}

struct Counting;

#[global_allocator]
static COUNTING: Counting = Counting;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static HIGH: AtomicUsize = AtomicUsize::new(0);

fn grow(bytes: usize) {
    let live = LIVE.fetch_add(bytes, Ordering::Relaxed) + bytes;
    HIGH.fetch_max(live, Ordering::Relaxed);
}

fn shrink(bytes: usize) {
    LIVE.fetch_sub(bytes, Ordering::Relaxed);
}

// SAFETY: every call goes straight to the system allocator; the counters only add up sizes.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        grow(layout.size());
        // SAFETY: the caller's contract, passed on.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        grow(layout.size());
        // SAFETY: the caller's contract, passed on.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        shrink(layout.size());
        // SAFETY: the caller's contract, passed on.
        unsafe { System.dealloc(ptr, layout) }
    }

    /// Counted as a move: the old and new blocks both live for a moment, as when the
    /// Worker's allocator can't grow a block in place.
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        grow(new_size);
        shrink(layout.size());
        // SAFETY: the caller's contract, passed on.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}
