//! Shared SDK fixtures read into the Rust protocol types and written back unchanged.

use knoa_core::protocol::{
    Account, McpSettings, McpSettingsResponse, OrganizeResponse, PullResponse, PushResponse,
};
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

#[test]
fn account_and_organize_answers_preserve_the_complete_contract() {
    #[derive(Deserialize)]
    struct Answers {
        accounts: Vec<Value>,
        organize: Vec<Value>,
    }
    let answers: Answers = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../sdk/fixtures/account-and-organize.json"
    )))
    .unwrap();
    for account in answers.accounts {
        round_trip::<Account>(&account);
    }
    for organize in answers.organize {
        round_trip::<OrganizeResponse>(&organize);
    }
}

#[test]
fn cached_accounts_without_billing_keep_their_existing_details() {
    let wire = serde_json::json!({
        "user_id": "user_legacy", "email": null, "plan": "pro", "organizer": "cloud",
        "limits": {"bookmarks": 10000, "organized_per_month": 1000},
        "usage": {"bookmarks": 12, "organized_this_month": 3},
    });
    let account: Account = serde_json::from_value(wire.clone()).unwrap();
    assert!(
        account.billing.is_none(),
        "unknown cached billing must not imply a free account"
    );
    assert_eq!(serde_json::to_value(account).unwrap(), wire);
}

fn round_trip<T: DeserializeOwned + Serialize>(wire: &Value) {
    let parsed: T = serde_json::from_value(wire.clone()).expect("the Rust type reads the answer");
    assert_eq!(&serde_json::to_value(&parsed).expect("it serializes"), wire);
}

#[test]
fn mcp_settings_preserve_the_shared_wire_contract() {
    let fixture: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../sdk/fixtures/mcp-settings.json"
    )))
    .unwrap();
    for settings in fixture["settings"].as_object().unwrap().values() {
        round_trip::<McpSettings>(settings);
        let mut response = settings.clone();
        response["url"] = fixture["url"].clone();
        round_trip::<McpSettingsResponse>(&response);
    }
}
