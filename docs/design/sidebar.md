# Sidebar (v2)

- Status: Built
- Date: 2026-11-16
- Prototype: the "Knoa sidebar explorations" design canvas, board "FINAL · Sidebar
  reference (T1 + B4 + M1)". It shows macOS, web and iPhone, the footer in every state
  and the account menu in every case. The later rounds on the canvas explain the choices.
- Replaces the Sidebar section of [main-screen.md](main-screen.md).

## Principles

- **One sidebar on every platform.** macOS and web have the same rows, controls, words and
  order. macOS only adds the traffic lights and a signed-out state. The iPhone uses the
  same parts, laid out the iOS way.
- **Space and speed over decoration.** Nothing here exists to look nice. The brand lives
  in the mascot and the colours, not in a wordmark.
- **Silence means fine.** Sync and the AI say nothing while all is well. When something
  needs saying, one card says it, with the action to fix it.
- **Grey line icons.** Every place uses the same muted line icon. Colour is for folders,
  the AI (brand red) and danger (wine).

## Anatomy

```
┌────────────────────────────┐
│ ● ● ●  [P] Personal ⇅    + │  titlebar row, 44 px
│ ▣ Everything            47 │
│ ▢ Gallery                7 │
│ ▢ Up next                  │  places
│ ▢ Unsorted              10 │
│ ◌ Not organized         12 │  only while bookmarks wait for the AI
│ ▢ Trash                  8 │  only while it holds something
│                            │
│ Folders                  + │  + shows on sidebar hover
│ ▢ Design                 6 │
│ ▢ Wallpapers             6 │
│     ▢ Anime              2 │
│ ┌────────────────────────┐ │
│ │ ◕ Organizing 12 of 47… │ │  status card, only when needed
│ │ ━━━━━━─────────────────│ │
│ └────────────────────────┘ │
├────────────────────────────┤
│ (TS) Tomasz Sz…  ⇅      ⚙ │  footer, 46 px
└────────────────────────────┘
```

## Panel

- Floating, 240 px wide, 12 px from the window edges (`SIDEBAR_WIDTH`, `MARGIN`).
- 16 px radius, `panel` fill, 1 px `hairline` border, the usual soft panel shadow.
- On web the panel sits the same way inside the page, below the browser chrome.

## Titlebar row (T1)

Height 44 px, padding 0 8 px; on macOS the left padding is 16 px and the traffic lights
come first (12 px dots, 8 px apart, 10 px before the next control).

- **Workspace button:** 28 px high, 8 px radius, padding 0 6 0 7 px, 8 px gap. It holds
  the workspace mark (18 px, 5 px radius, the workspace colour, its initial in 10.5 px
  bold), the name (13 px semibold, ellipsis) and a 12 px up-down chevron in `muted`.
  Hover: `hover` fill. Clicking opens the workspace switcher, unchanged from
  main-screen.md.
- **Add link (+):** a 28 px icon button pushed to the right edge, 8 px radius, 16 px icon
  in `muted`, `text` on hover. ⌘N. Tooltip "Add link (⌘N)".
- Web: the same row without the traffic lights, so the workspace button starts at 8 px.

## Rows (places and folders)

- 30 px high, 1 px apart, 8 px radius. The list has 2 px top and 8 px side padding.
  The shared `ROW_HEIGHT` (32) stays for other lists.
- Content: padding 0 10 0 8 px, 10 px gap; a 16 px line icon (1.75 stroke), the label
  (13 px, ellipsis) and the count (12 px, `muted`, tabular figures, right-aligned, at
  least 20 px wide). A zero count shows nothing.
- Hover: `hover` fill. Selected: the selected-row fill, label weight 500, and a place's
  icon turns `accent`. Folder icons keep their folder colour.
- No ⌘-number hints in rows. ⌘1–9 still work.

### Places

Everything (layers), Gallery (image), Up next (bookmark), Unsorted (inbox), Not organized
(dashed circle), Trash (trash), Set aside (when its rule applies, as today).

- **Not organized** shows while any bookmark is waiting for the AI. Its icon is the
  Lucide dashed circle in `muted` like the rest, with a `muted` count: no red dot. While
  the AI is organizing, the circle turns slowly (one turn per 8 s). It stays still with
  reduced motion.

### Folders

- Section header: 28 px high, 18 px above it, padding 0 5 0 8 px, "Folders" in 11.5 px
  semibold `muted`, sentence case. A 22 px + button (6 px radius, 14 px icon) at its
  right end appears while the pointer is over the sidebar: New folder, ⇧⌘N.
- There is no "New folder" row.
- Nesting: each level indents the row by 26 px (level 1 starts at 34 px), so a child's
  icon sits under its parent's label. Three levels at most.
- Hovering a parent folder swaps its icon for a 24 px disclosure chevron (14 px icon)
  that rotates 90° when open. Clicking the chevron opens or closes; clicking the row
  selects it.
- Hovering any folder swaps its count for a 24 px ⋯ button (the folder menu) in the same
  place, so counts never move.

## Status card (B4)

The only place the sidebar reports the AI or sync. Shown above the footer only when there
is something to say; otherwise there is nothing there.

- 8 px from the panel sides and from the footer, padding 9 8 9 9 px, 10 px radius,
  `soft` fill.
- One row, at least 24 px high, 8 px gap: the mascot (18 px) or a 16 px sync icon, the
  message (12.5 px) and an optional action (12 px semibold `accent`, 24 px high, padding
  0 6 px, 6 px radius, `hover` on hover).
- While organizing, a 3 px progress bar below the row: `hairline` track, `ai` fill,
  2 px radius.

One message at a time, most urgent first:

| State | Icon | Message | Tone | Action |
| --- | --- | --- | --- | --- |
| AI needs attention | Mascot, worried | AI needs attention | `danger` | Fix |
| Sync stopped | Cloud with alert, `danger` | Sync stopped | `danger` | Retry |
| Sync paused (macOS, library linked to another account) | Cloud with alert, `danger` | Sync paused | `danger` | Fix |
| Some changes didn't sync (the server refused them) | Cloud with alert, `danger` | Changes didn't sync | `text` | View |
| Plan limit reached | Cloud with alert, `danger` | Plan limit reached | `text` | Upgrade |
| Organizing | Mascot, organizing | Organizing 12 of 47… | `ai_text` | none, progress bar |
| AI off | Mascot, asleep (grey) | AI is off | `text` | Turn on |
| Offline | Cloud off, `muted` | Offline. Changes sync later. | `text` | none |
| Anything else | | (no card) | | |

Sync states apply only when signed in. "Retry" retries sync. "Fix" on Sync paused and
"View" open Settings at the Account page, which says what happened. "Upgrade" opens
the upgrade page. "Turn on" opens the AI settings: the AI is off only while no provider
is set up, so setting one up is how it turns on.

AI needs attention covers two causes. When the AI itself fails, "Fix" opens the AI
settings. When the cloud organizes and this month's organizing allowance is used up,
"Fix" opens Settings at the Account page.

"Organizing N of M…" counts the current batch, not the library: the batch starts when
bookmarks begin waiting, grows as more arrive, and ends when none wait; N is the one being
worked on and the bar shows the share done.

## Footer (B4)

Height 46 px, padding 0 8 px, a 1 px `hairline` line on top. Two controls:

- **Account button**, filling the width: 32 px high, 10 px radius, padding 0 8 0 5 px,
  8 px gap. A 22 px round avatar (initials in 9 px bold), the name (13 px semibold,
  ellipsis) and a 12 px up-down chevron in `muted`. Hover: `hover` fill. While its menu
  is open: the selected-row fill. Clicking toggles the account menu.
- **Settings (gear):** a 28 px icon button at the right. ⌘,. Tooltip "Settings (⌘,)".

Signed out (macOS only): the avatar becomes a 22 px rounded square with a laptop icon
and the name reads "This Mac". The web is always signed in.

## Account menu (M1)

A popover that opens upward from the account button.

- 224 px wide, the width of the rows, left-aligned with them; its bottom edge 4 px above
  the footer line. 12 px radius, 5 px padding, `panel` fill, a 1 px `hairline` ring and a
  soft drop shadow.
- Items: 30 px high, 7 px radius, padding 0 8 px, 9 px gap, a 15 px `muted` icon and 13 px
  text. Hover: `hover` fill. Separators: 1 px `hairline`, 4 px above and below, inset
  6 px.
- Closes on Esc, on a click outside, when focus moves elsewhere, and after choosing an
  item.

Signed in (macOS and web):

1. **Header** (padding 8 8 7 px, 10 px gap): 28 px avatar, the name (13 px semibold) and
   the email (11.5 px `muted`). Not clickable.
2. Separator.
3. **Sync line**: not a button. A 15 px icon and 12.5 px text, 30 px high:
   - "Synced just now" with a cloud-check icon, `muted`; then "1 min ago", "1 hr ago",
     kept current while the menu is open
   - "Offline · 3 changes waiting" with cloud-off, `muted`
   - "Sync stopped" with cloud-alert, `danger`, and a Retry action at the right
4. Separator.
5. **Manage plan…** (credit card icon) on Pro, or **Upgrade to Pro** (zap icon) on Free.
   Both are ordinary items: Free gets no busier menu than Pro.
6. **Account settings…** (person icon): opens Settings at the Account page.
7. **Open Knoa on the web** (external link icon) on macOS, or **Get Knoa for Mac**
   (laptop icon) on the web.
8. Separator.
9. **Sign out** (log-out icon). The web keeps its existing confirm when changes are
   unsent.

Signed out (macOS only): one ordinary item, **Sign in…** (log-in icon). No header, no
explanation and no separator; the footer button already reads "This Mac". While waiting
for the browser, that same item shows a turning dashed circle, "Finish in browser" and a
small Cancel action at its right end.

The menu has no workspaces (the titlebar owns them), no AI switch (the status card's
"Turn on" covers it, and the rest is in Settings) and no Settings item (the gear is
right next to it).

## iPhone

The sidebar is the home screen. The same parts, laid out the iOS way:

- Top-right: a 32 px avatar with the mascot as a small badge. It opens the **account
  sheet**.
- Large title "Personal ⌄" (32 px bold) opens the workspace switcher. Under it, the
  status card's message as one line (mascot and text, with its action), only when there
  is something to say.
- Places, then "Folders", as inset grouped lists with 48 px rows, 20 px `muted` icons,
  counts and chevrons.
- Add link: a 56 px round floating button at the bottom right, `text` fill and `panel`
  icon.
- **Account sheet:** a sheet with a grabber, the title "Account" and Done. Groups, in
  order: profile (46 px avatar, name, "email · Knoa Pro"); the sync line (with Retry
  when stopped); Manage plan, Account settings and **Settings** (the phone has no gear,
  so Settings lives here); Sign out, centred, in `danger`.

## What goes away

- The footer wordmark, the faint "AI off" text and the always-on cloud icon.
- The red dot as the Not organized icon.
- The "New folder" row, and the ⌘-number hints on folder rows.
- On web: the workspace row (it moves into the top row), the uppercase "FOLDERS" label,
  the Settings row and the account row with the plan.

## Copy

| Where | Text |
| --- | --- |
| Add link tooltip | Add link (⌘N) |
| New folder tooltip | New folder (⇧⌘N) |
| Settings tooltip | Settings (⌘,) |
| Status card | AI needs attention · Sync stopped · Sync paused · Changes didn't sync · Plan limit reached · Organizing 12 of 47… · AI is off · Offline. Changes sync later. |
| Card actions | Fix · Retry · View · Upgrade · Turn on |
| Menu | Synced just now · Offline · 3 changes waiting · Sync stopped · Manage plan… · Upgrade to Pro · Account settings… · Open Knoa on the web · Get Knoa for Mac · Sign out |
| Signed out | This Mac (footer) · Sign in… · Finish in browser · Cancel |

## Web differences

- No "AI is off" state: the server organizes, and the web has no AI switch.
- No ⌘ shortcuts: the browser keeps ⌘N, ⌘, and ⌘1–9, so web tooltips name no shortcut.

## Rejected during exploration

| Idea | Why not |
| --- | --- |
| A workspace rail (Slack, Discord) | Most people keep one workspace; a rail is mostly empty |
| Places as tiles, an editorial index, a top identity card | Look good, but cost space; a daily tool needs density more |
| Wordmark or logo in the sidebar | Decoration nobody needs after day one |
| An add-link field instead of + | Costs a row; ⌘N and + are enough |
| Settings inside the account menu, or as its own row | One more click, or one more row; the gear is clearer |
| Status line always on | Noise while all is well; the card speaks only when needed |
| AI switch in the account menu | Redundant with the status card's Turn on and with Settings |
| Account card or status tiles in the menu | The plain menu is shorter, and the status card already reports problems |
