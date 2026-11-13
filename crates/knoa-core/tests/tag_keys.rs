//! The SDK's tag fixture, which both cores must key the same way.

use knoa_core::protocol::tag_key;
use knoa_core::{normalize_tag, tag_fits};
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    bookmark: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    input: String,
    /// `None` when nothing protocol-sized is left of the tag.
    key: Option<String>,
}

#[test]
fn every_tag_in_the_fixture_gets_its_key() {
    let fixture: Fixture = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../sdk/fixtures/tag-keys.json"
    )))
    .unwrap();
    for case in fixture.cases {
        let tag = normalize_tag(&case.input);
        let key = tag_fits(&tag).then(|| tag_key(&fixture.bookmark, &tag));
        assert_eq!(key, case.key, "input {:?}", case.input);
    }
}
