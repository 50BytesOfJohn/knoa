//! AI access settings and optimistic changes, independent of library sync.

use knoa_core::protocol::{McpAccess, McpSettings, McpSettingsResponse};

/// A connection's settings, keeping the previous values until a save succeeds.
#[derive(Debug, Default)]
pub struct McpState {
    settings: Option<McpSettingsResponse>,
    operation: Operation,
    problem: Option<String>,
}

#[derive(Debug, Default)]
enum Operation {
    #[default]
    Idle,
    Loading,
    Saving(McpSettingsResponse),
}

pub(super) enum Change {
    Default(McpAccess),
    Workspace(String, Option<McpAccess>),
}

impl McpState {
    pub fn settings(&self) -> Option<&McpSettingsResponse> {
        self.settings.as_ref()
    }

    pub fn busy(&self) -> bool {
        !matches!(self.operation, Operation::Idle)
    }

    pub fn saving(&self) -> bool {
        matches!(self.operation, Operation::Saving(_))
    }

    pub fn problem(&self) -> Option<&str> {
        self.problem.as_deref()
    }

    pub(super) fn begin_load(&mut self) -> bool {
        if self.busy() {
            return false;
        }
        self.operation = Operation::Loading;
        self.problem = None;
        true
    }

    pub(super) fn begin_save(&mut self, change: Change) -> Option<McpSettings> {
        if self.busy() {
            return None;
        }
        let previous = self.settings.as_ref()?;
        let mut next = previous.clone();
        match change {
            Change::Default(access) => next.access = access,
            Change::Workspace(id, Some(access)) => {
                next.workspaces.insert(id, access);
            }
            Change::Workspace(id, None) => {
                next.workspaces.remove(&id);
            }
        }
        if &next == previous {
            return None;
        }
        let body = McpSettings {
            access: next.access,
            workspaces: next.workspaces.clone(),
        };
        self.operation = Operation::Saving(previous.clone());
        self.settings = Some(next);
        self.problem = None;
        Some(body)
    }

    pub(super) fn finish(&mut self, result: Result<McpSettingsResponse, String>) {
        let operation = std::mem::take(&mut self.operation);
        match result {
            Ok(settings) => self.settings = Some(settings),
            Err(problem) => {
                if let Operation::Saving(previous) = operation {
                    self.settings = Some(previous);
                }
                self.problem = Some(problem);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loaded() -> McpState {
        let mut state = McpState::default();
        state.begin_load();
        state.finish(Ok(McpSettingsResponse {
            access: McpAccess::Read,
            workspaces: [
                ("private".into(), McpAccess::Off),
                ("work".into(), McpAccess::Read),
            ]
            .into(),
            url: "https://api.example.com/mcp".into(),
        }));
        state
    }

    #[test]
    fn failed_save_restores_access_and_overrides_and_allows_retry() {
        let mut state = loaded();
        let previous = state.settings().cloned();
        state.begin_save(Change::Workspace("private".into(), Some(McpAccess::Full)));
        assert!(state.saving());
        assert_eq!(
            state.settings().unwrap().workspaces["private"],
            McpAccess::Full
        );

        state.finish(Err("Offline".into()));
        assert_eq!(state.settings(), previous.as_ref());
        assert_eq!(state.problem(), Some("Offline"));
        assert!(!state.busy());
        assert!(state.begin_save(Change::Default(McpAccess::Off)).is_some());
        assert_eq!(state.problem(), None);
        state.finish(Err("Offline".into()));
        assert_eq!(state.settings(), previous.as_ref());
    }

    #[test]
    fn following_default_removes_only_that_override_and_uses_the_server_answer() {
        let mut state = loaded();
        let body = state
            .begin_save(Change::Workspace("private".into(), None))
            .unwrap();
        assert!(!body.workspaces.contains_key("private"));
        assert_eq!(body.workspaces["work"], McpAccess::Read);
        let mut answer = state.settings().unwrap().clone();
        answer.url = "https://another.example.com/mcp".into();
        state.finish(Ok(answer.clone()));
        assert_eq!(state.settings(), Some(&answer));

        let body = state.begin_save(Change::Default(McpAccess::Full)).unwrap();
        assert_eq!(body.workspaces["work"], McpAccess::Read);
    }

    #[test]
    fn whole_setting_requests_do_not_overlap() {
        let mut state = McpState::default();
        assert!(state.begin_save(Change::Default(McpAccess::Full)).is_none());
        assert!(state.begin_load());
        assert!(!state.begin_load());
        assert!(state.begin_save(Change::Default(McpAccess::Full)).is_none());
        state.finish(Err("Offline".into()));
        assert!(!state.busy());
        assert!(state.begin_load());

        state = loaded();
        assert!(state.begin_save(Change::Default(McpAccess::Read)).is_none());
        assert!(state.begin_save(Change::Default(McpAccess::Off)).is_some());
        assert!(!state.begin_load());
        assert!(
            state
                .begin_save(Change::Workspace("work".into(), None))
                .is_none()
        );
    }
}
