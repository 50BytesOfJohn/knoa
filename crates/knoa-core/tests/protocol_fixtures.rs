//! Shared SDK fixtures read into the Rust protocol types and written back unchanged.

use knoa_core::protocol::{PullResponse, PushResponse};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

#[derive(Deserialize)]
struct Fixture {
    steps: Vec<Step>,
}

#[derive(Deserialize)]
struct Step {
    push: Option<Value>,
    pull: Option<Value>,
    expect: Value,
}

#[test]
fn every_server_answer_in_the_fixtures_round_trips() {
    let files = [
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../sdk/fixtures/create-and-pull.json"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../sdk/fixtures/tombstone-revive.json"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../sdk/fixtures/user-beats-ai.json"
        )),
    ];
    for file in files {
        let fixture: Fixture = serde_json::from_str(file).unwrap();
        for step in fixture.steps {
            match (step.push, step.pull) {
                (Some(_), None) => round_trip::<PushResponse>(&step.expect),
                (None, Some(_)) => round_trip::<PullResponse>(&step.expect),
                _ => panic!("a step is a push or a pull"),
            }
        }
    }
}

fn round_trip<T: DeserializeOwned + Serialize>(wire: &Value) {
    let parsed: T = serde_json::from_value(wire.clone()).expect("the Rust type reads the answer");
    assert_eq!(&serde_json::to_value(&parsed).expect("it serializes"), wire);
}
