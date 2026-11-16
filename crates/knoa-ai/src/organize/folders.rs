//! Folder matching with a yes/no classifier: one question per auto-assign folder, each
//! asked on its own whatever its parent's answer, and the most specific match kept.

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
    let scored = input.folders.iter().map(|f| f.id).zip(probabilities);
    Ok(deepest(&choose(scored.collect()), input.folders))
}

/// The picks without any folder that has another pick under it: filed into "Wallpapers"
/// and "Wallpapers › Anime", a bookmark goes into Anime alone, and still shows under
/// Wallpapers. A parent with no picked subfolder keeps its pick.
pub(super) fn deepest(picks: &[(FolderId, f32)], folders: &[FolderInput]) -> Vec<(FolderId, f32)> {
    let is_above_a_pick = |id: FolderId| {
        picks.iter().any(|(pick, _)| {
            folders
                .iter()
                .find(|f| f.id == *pick)
                .is_some_and(|f| f.ancestors.iter().any(|a| a.id == id))
        })
    };
    picks
        .iter()
        .copied()
        .filter(|&(id, _)| !is_above_a_pick(id))
        .collect()
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
    let mut about = if folder.description.trim().is_empty() {
        folder.name.clone()
    } else {
        folder.description.trim().to_owned()
    };
    let ancestry = folder.ancestry();
    if !ancestry.is_empty() {
        about = format!("{about} ({ancestry})");
    }
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

    #[test]
    fn a_pick_and_a_pick_under_it_keep_only_the_deeper_one() {
        let folder = |id, above: &[i64]| FolderInput {
            id: FolderId(id),
            name: format!("f{id}"),
            description: String::new(),
            ancestors: above
                .iter()
                .map(|&a| crate::FolderAncestor {
                    id: FolderId(a),
                    name: format!("f{a}"),
                    description: String::new(),
                })
                .collect(),
        };
        // 1 › 2 › 3, and 4 on its own.
        let folders = [
            folder(1, &[]),
            folder(2, &[1]),
            folder(3, &[1, 2]),
            folder(4, &[]),
        ];

        let picked = deepest(&[(FolderId(1), 0.95), (FolderId(3), 0.9)], &folders);
        assert_eq!(picked, [(FolderId(3), 0.9)]);
        let unrelated = deepest(&[(FolderId(1), 0.95), (FolderId(4), 0.9)], &folders);
        assert_eq!(unrelated, [(FolderId(1), 0.95), (FolderId(4), 0.9)]);
    }
}
