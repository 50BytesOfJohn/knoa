# Pull requests

## Before opening

- Branch off `main`. One change per PR. A preparatory refactor can be its own commit.
- `make check` passes, and the change was verified in the app (verifying.md).

## Title

What changed, in plain words, under 60 characters. `Import Arc bookmarks`, not
`feat: implement Arc browser bookmark ingestion pipeline`.

## Description

Write for someone who has 30 seconds. Say what matters, skip the rest: the diff has the
details.

```markdown
<What changes for the user or the codebase, in one or two sentences.>

<Why, if the title doesn't make it obvious.>

<Screenshot, diagram, or table if it explains faster than words.>

<Anything the reviewer must know: a tradeoff, a follow-up, what you couldn't verify.>
```

Leave out:

- a file-by-file list of changes
- "Summary", "Changes", "Testing" headers on a short PR
- restating the title
- tests run, unless something unusual was checked

## Visuals

Use one when it beats a paragraph.

- UI change: a screenshot, required. Before and after side by side if a screen changed.
- Data flow or architecture: an ASCII diagram in a code block.
- Numbers (sizes, timings, counts): a small table.

Host screenshots on the `pr-assets` orphan branch, never on the PR branch:

```bash
git worktree add /tmp/pr-assets pr-assets   # first time: git worktree add --orphan -b pr-assets /tmp/pr-assets
cp shot.png /tmp/pr-assets/<branch>-<name>.png
git -C /tmp/pr-assets add . && git -C /tmp/pr-assets commit -m "<branch> screenshots" && git -C /tmp/pr-assets push -u origin pr-assets
git worktree remove /tmp/pr-assets
```

Reference it as `![<what it shows>](https://raw.githubusercontent.com/50BytesOfJohn/knoa/pr-assets/<branch>-<name>.png)`.

## Writing

From [unslop](https://raw.githubusercontent.com/cursor/plugins/refs/heads/main/pstack/skills/unslop/SKILL.md):

- Plain words. "use" not "leverage", "is" not "serves as".
- Say what it does, with names and numbers, not how it feels.
- Active voice, short sentences, one idea each. Full sentences, no arrow-speak.
- No em dashes, decorative emojis, bold everywhere, or title case headings.
- No hedging, no filler ("In order to", "It's worth noting"), no generic closing line.
- No AI vocabulary: crucial, robust, seamless, comprehensive, enhance, ensure, delve.
