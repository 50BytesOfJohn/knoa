//! The wire format: one organize job in, its result out. Field names are the contract
//! with the API Worker, so they live here and nowhere else.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One bookmark to organize, as queued by the API Worker.
#[derive(Debug, Deserialize)]
pub struct Job {
    pub user_id: String,
    pub bookmark_id: String,
    pub url: String,
    pub title: String,
    pub browser_folder: Option<String>,
    pub workspace: Workspace,
    /// The workspace's folders with auto-assign on.
    #[serde(default)]
    pub folders: Vec<Folder>,
    /// File an organized bookmark again from `existing`, without fetching the page.
    #[serde(default)]
    pub folders_only: bool,
    pub existing: Option<Existing>,
    pub api_key_source: ApiKeySource,
}

#[derive(Debug, Deserialize)]
pub struct Workspace {
    #[serde(default)]
    pub instructions: String,
    pub guideline: Option<String>,
    #[serde(default)]
    pub tag_list: Vec<ListedTag>,
    #[serde(default)]
    pub only_listed_tags: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListedTag {
    pub tag: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
}

/// What an earlier run stored, for a folders-only job.
#[derive(Debug, Deserialize)]
pub struct Existing {
    pub kind: String,
    pub summary: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeySource {
    /// Knoa's own provider key, for subscribers.
    Ours,
}

/// What organizing found. A field left out means "keep what is stored": the same rules
/// the desktop store applies when it saves a page and then the AI's answer.
#[derive(Debug, Default, Serialize)]
pub struct JobResult {
    pub bookmark_id: String,
    /// The page's own title, only for a bookmark whose title is still its URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub site: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// The page's own description, the summary to show while there is no AI summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub fields: serde_json::Map<String, Value>,
    pub extras: Vec<Extra>,
    pub tags: Vec<String>,
    pub folders: Vec<FolderPick>,
    /// Why the bookmark doesn't fit the workspace's guideline, when it doesn't.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_aside: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<Image>,
    /// Readable page text, for search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_text: Option<String>,
    /// The fetch's evidence JSON, kept so organizing again needs no refetch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_evidence: Option<String>,
    /// The model that answered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Also on failed jobs: answers that turned out unusable are billed too.
    pub usage: Vec<ModelUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Extra {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct FolderPick {
    pub id: String,
    /// The classifier's probability; 1 when a chat model picked the folder.
    pub probability: f32,
}

/// The preview image's source. Not stored anywhere yet.
#[derive(Debug, Serialize)]
pub struct Image {
    pub url: String,
    pub width: u32,
    pub height: u32,
}

/// One model's requests in this job, summed like the desktop's daily `ai_usage` rows.
#[derive(Debug, Serialize)]
pub struct ModelUsage {
    pub provider: String,
    pub model: String,
    pub requests: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    /// In US dollars.
    pub cost: f64,
    /// Requests the provider reported no token counts for.
    pub unmetered: u64,
    /// Requests the provider reported no cost for.
    pub unpriced: u64,
}
