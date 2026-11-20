//! Rebuilds when a translation changes, so `fl!` checks message IDs against the new files.

fn main() {
    println!("cargo:rerun-if-changed=i18n");
}
