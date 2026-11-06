//! Folder matching with a yes/no classifier: one question per auto-assign folder.

use knoa_core::{FolderId, Kind};
use serde_json::{Value, json};

use super::FolderInput;
use crate::{Classifier, Error, Question};

/// A folder needs at least this probability to be picked.
const MIN_PROBABILITY: f32 = 0.7;
/// A second folder is picked only when it's this close to the best one.
const SECOND_WITHIN: f32 = 0.1;

/// What the classifier sees of a bookmark: what organizing it found, not the page.
pub struct FilingInput<'a> {
    pub title: &'a str,
    pub url: &'a str,
    pub browser_folder: Option<&'a str>,
    pub kind: Kind,
    pub summary: Option<&'a str>,
    pub tags: &'a [String],
    /// The workspace's organizing instructions.
    pub instructions: &'a str,
    /// Folders with auto-assign on.
    pub folders: &'a [FolderInput],
}

/// Picks the bookmark's folders, each with its probability. Needs only the classifier, so
/// filing an organized bookmark again costs one request.
pub async fn file_into_folders<C: Classifier>(
    classifier: &C,
    input: &FilingInput<'_>,
) -> Result<Vec<(FolderId, f32)>, Error> {
    if input.folders.is_empty() {
        return Ok(Vec::new());
    }
    let questions: Vec<Question> = input.folders.iter().map(question).collect();
    let probabilities = classifier.classify(&state(input), &questions).await?;
    Ok(choose(
        input
            .folders
            .iter()
            .map(|f| f.id)
            .zip(probabilities)
            .collect(),
    ))
}

/// Raw probabilities run high across the board, so the rule is rank-based: the best folder
/// if it's confident, plus a runner-up that is nearly as good.
fn choose(mut scored: Vec<(FolderId, f32)>) -> Vec<(FolderId, f32)> {
    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
    let Some(&(_, best)) = scored.first() else {
        return Vec::new();
    };
    scored
        .into_iter()
        .take(2)
        .filter(|&(_, p)| p >= MIN_PROBABILITY && best - p <= SECOND_WITHIN)
        .collect()
}

fn question(folder: &FolderInput) -> Question {
    let about = if folder.description.trim().is_empty() {
        folder.name.clone()
    } else {
        folder.description.trim().to_owned()
    };
    Question {
        key: format!("folder_{}", folder.id.0),
        instructions: format!(
            "Does this bookmark belong in the user's folder \"{}\"?",
            folder.name
        ),
        yes: format!("It clearly fits the folder: {about}"),
        no: "It does not belong in this folder.".into(),
    }
}

fn state(input: &FilingInput<'_>) -> Value {
    json!({
        "bookmark": {
            "title": input.title,
            "url": super::prompt::redact(input.url),
            "browser_folder": input.browser_folder,
            "kind": input.kind.slug(),
            "summary": input.summary,
            "tags": input.tags,
        },
        "user_instructions": input.instructions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_a_confident_best_folder_and_a_close_runner_up() {
        let picked = choose(vec![
            (FolderId(1), 0.6),
            (FolderId(2), 0.92),
            (FolderId(3), 0.85),
        ]);
        assert_eq!(picked, [(FolderId(2), 0.92), (FolderId(3), 0.85)]);
        assert_eq!(choose(vec![(FolderId(1), 0.65)]), []);
    }
}
