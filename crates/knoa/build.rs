//! Rebuild when the distribution channel changes.

fn main() {
    println!("cargo:rerun-if-env-changed=KNOA_CHANNEL");
}
