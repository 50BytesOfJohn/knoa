# 0011: Workspaces are fed by browsers, with an optional guideline

- Status: Accepted
- Date: 2026-11-04

## Context

Workspaces were planned for after the alpha: separate libraries such as Work and
Personal, each with its own folders and instructions. The open question was how a
bookmark gets into a workspace. Letting the AI route each bookmark between workspaces
makes every bookmark depend on a guess across all of them, and a wrong guess hides it in
a library the user isn't looking at.

## Decision

- A workspace is a row in the database with its name, colour, organizing instructions
  and the browser profiles it syncs. Folders belong to one workspace.
- The browser decides: every bookmark from a profile goes into each workspace that
  syncs it. A profile can feed several workspaces; each keeps its own copy, so URLs are
  unique per workspace, not globally.
- There is no routing between workspaces. A workspace can add an optional guideline
  ("only things about work"), off by default. With it on, the organizing call also
  answers whether the bookmark fits, with a reason. A bookmark that doesn't fit is
  organized as usual but set aside: it shows only in a "Set aside" place until the user
  keeps it, and a kept bookmark is never set aside again.
- Links can be added by hand (⌘N). They skip the guideline, since the user chose them.
- Pre-workspace settings (name, colour, browsers, instructions) move into the first
  workspace on the first launch after the migration.

## Consequences

- Splitting one browser into two workspaces works by giving both the profile and each a
  guideline. Each decides on its own, which costs one organizing call per workspace.
- The guideline adds two fields to the existing chat call, not a new request.
- The schema migration rebuilds the `bookmarks` table to change its unique key, with
  foreign keys off during migrations so the rebuild doesn't cascade.
- Hidden workspaces are not built yet.
