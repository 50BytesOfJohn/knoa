# 0017: Folders nest three levels deep, and an icon can be an emoji

- Status: Accepted
- Date: 2026-11-16
- Supersedes: [0005](0005-typed-bookmarks-and-folders.md) (one-level folders only; the
  rest of 0005 stands)

## Context

Users with many folders want to group them ("Wallpapers" with "Anime" and "Cities" under
it) and to tell them apart at a glance. One level and 28 glyphs did neither. Released
apps (Mac 0.2.2) and the API predate nesting, so the change has to keep them working.

## Decision

- **Model:** a folder has an optional parent in the same workspace, at most three levels
  deep (`MAX_FOLDER_DEPTH`). Folders stay a flat list with a parent reference; no new
  entity. A bookmark can still be in several folders, a parent included.
- **One tree module per language:** `FolderTree` in `knoa-core` and its mirror in the
  SDK. It gives display order with depth, children, subtrees, paths and move checks
  (cycle, depth, workspace), and builds a valid tree from any rows: a folder whose parent
  is missing, cyclic, in another workspace or too deep shows at the top level.
- **A parent shows and counts its subtree:** selecting it shows its own bookmarks and its
  descendants', and its count is the number of distinct bookmarks in its subtree.
  Unsorted is still "in no folder". Counts come from one query fed the tree's
  (ancestor, folder) pairs.
- **Siblings sort by `position`;** a new or moved folder goes last among its siblings.
- **Deleting** a folder from our apps deletes its subtree, one recorded delete per folder,
  deepest first. Bookmarks only lose those memberships. The local `parent_id` doesn't
  cascade.
- **Wire:** the folder row gains a nullable `parent_id`. A whole-row `put` leaves it out
  at the top level, because an API from before nesting rejects unknown columns and
  clients drop rejected mutations for good; a move back to the top level sends an
  explicit null. The API deploys before clients that can set parents.
- **The server owns integrity:** it never stores a cycle, a missing or cross-workspace
  parent, or a deeper tree, and when a folder is deleted it moves the folder's children
  to its parent with new versions, so 0.2.2 can't strand them.
- **Pulls** may bring a child before its parent; applying loops until the page settles.
  Linking a second device adopts folders by workspace and path of names, parents first.
- **Icons** stay one text column of at most 100 characters: a known glyph name is that
  glyph, a single emoji (one grapheme cluster) is that emoji, anything else is the folder
  shape. Only glyphs take the folder's color.
- **AI filing** still asks about each auto-assign folder, named by its full path.
- Free for everyone: no plan checks.

## Consequences

- A user without subfolders sees no change; hierarchy shows only where there is some.
- 0.2.2 shows nested folders flat and can't nest, but stays in step with the account.
- Lists that show folders flat (menus, chips, the search scope) name a subfolder by its
  path, "Wallpapers › Anime".
- Every client keeps the same tree rules; `sdk/fixtures/nested-folders.json` pins the
  sync side of them.
