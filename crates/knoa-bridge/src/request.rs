//! What the extension can ask for: the workspaces, and saving a link to one of them.

use std::os::unix::net::UnixDatagram;

use anyhow::{Context as _, bail};
use knoa_core::{WorkspaceId, parse_link};
use knoa_paths::Paths;
use knoa_store::{AddOutcome, NewLink, Store};
use serde::Deserialize;
use serde_json::{Value, json};

/// The extension cuts snapshots down to this; the API refuses larger ones too.
const MAX_SNAPSHOT_BYTES: usize = 1024 * 1024;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Request {
    Hello,
    Save {
        url: String,
        title: Option<String>,
        workspace: Option<i64>,
        snapshot: Option<String>,
    },
}

/// Answers one request. Failures become `{ "ok": false, "error": ... }` so the extension
/// can tell the user instead of seeing the host exit.
pub fn handle(message: Value) -> Value {
    match answer(message) {
        Ok(reply) => reply,
        Err(err) => json!({ "ok": false, "error": format!("{err:#}") }),
    }
}

fn answer(message: Value) -> anyhow::Result<Value> {
    let request: Request = serde_json::from_value(message).context("unknown request")?;
    let paths = Paths::new()?;
    let mut store = Store::open(&paths.database()).context("opening the library")?;
    match request {
        Request::Hello => {
            let workspaces: Vec<Value> = store
                .workspaces()?
                .into_iter()
                .map(|w| json!({ "id": w.id.0, "name": w.name }))
                .collect();
            Ok(json!({
                "ok": true,
                "version": env!("CARGO_PKG_VERSION"),
                "workspaces": workspaces,
            }))
        }
        Request::Save {
            url,
            title,
            workspace,
            snapshot,
        } => {
            let Some(url) = parse_link(&url) else {
                bail!("only web pages can be saved");
            };
            if snapshot
                .as_ref()
                .is_some_and(|s| s.len() > MAX_SNAPSHOT_BYTES)
            {
                bail!("the page content is over {MAX_SNAPSHOT_BYTES} bytes");
            }
            let workspace = pick_workspace(&store, workspace)?;
            let link = NewLink {
                url: &url,
                title: title.as_deref(),
                snapshot: snapshot.as_deref(),
            };
            let now = jiff::Timestamp::now().as_second();
            let added = store.add_link(workspace, &link, now)?;
            poke_app(&paths);
            Ok(json!({ "ok": true, "outcome": outcome_name(added.outcome) }))
        }
    }
}

/// The workspace the extension picked, or the first one.
fn pick_workspace(store: &Store, wanted: Option<i64>) -> anyhow::Result<WorkspaceId> {
    let workspaces = store.workspaces()?;
    let found = match wanted {
        Some(id) => workspaces.iter().find(|w| w.id.0 == id),
        None => workspaces.first(),
    };
    Ok(found.context("no such workspace")?.id)
}

fn outcome_name(outcome: AddOutcome) -> &'static str {
    match outcome {
        AddOutcome::Added => "added",
        AddOutcome::Restored => "restored",
        AddOutcome::Existing => "existing",
    }
}

/// Tells a running Knoa the library changed. Nobody listening means Knoa is closed and
/// will find the link in its queue at launch.
fn poke_app(paths: &Paths) {
    if let Ok(socket) = UnixDatagram::unbound() {
        let _ = socket.send_to(b"changed", paths.bridge_socket());
    }
}
