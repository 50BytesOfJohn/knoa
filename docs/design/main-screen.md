# Main screen (v1)

- Status: Approved for the alpha
- Date: 2026-11-04
- Prototype: the "Knoa main screen explorations" design canvas, board "1.1 · Baseline"
  (layout) and "1.1 · Default, no wallpaper" (default look). The prototype is a
  reference for layout and behaviour, not for exact colours.
- Background: [research report](../research/main-screen.md)

## Principles

- **The library is visible the moment the app opens.** No click, no empty search screen.
  The fastest path is: open Knoa, see recent bookmarks, open one.
- **Search is one keystroke away.** Typing anywhere goes to the search field.
- **Familiar layout, our own look.** A sidebar, a content area and a detail panel, the
  layout every macOS user already knows. The character comes from the look: floating
  panels, wallpaper, type and cards.
- **Calm.** No visual noise on cards, no highlight on the selected card. One control for
  view options, one for settings.

## Layout

```
┌─────────────┐   ┌──────────── search pill ────────────┐        ┌─────────────┐
│ ● ● ●       │   │ ⌕ Just start typing…    12 found  ≡ │        │ detail      │
│ [P] Personal│   └─────────────────────────────────────┘        │ panel       │
│     13  ⇕   │   All · Movies · Articles · Video · Code …       │ (floating)  │
│             │                                                  │             │
│ Everything  │   ┌ Up next · 5 to read and watch ─────────── ✕ ┐│             │
│ Up next     │   └──────────────────────────────────────────────┘│             │
│ Unsorted    │   Today                                       ─  │             │
│ Not organized│  ┌────┐ ┌────┐ ┌────┐                        ─  │             │
│             │   │    │ │    │ │    │   masonry cards        ─  │             │
│ Folders     │   └────┘ │    │ └────┘                        ─  │             │
│  ▢ Watch…   │   Yesterday                          date rail ─  │             │
│  ▢ Reading  │                                                  │             │
│ + New folder│                                                  │ [Open ↵] ▮ ⧉│
│ Knoa  ✦1  ⚙ │                                                  └─────────────┘
└─────────────┘
```

All panels float over the window background with a 12 px margin and large corner radii.
The window background is a solid colour by default, or a wallpaper the user picks.

### Sidebar (left, floating, ~232 px)

1. **Titlebar:** the traffic lights on the left; on the right the **workspace button**
   (the workspace's coloured mark and a chevron, its name on hover) and **add link**.
   The workspace stays small because most people keep one; the bookmark count lives on
   Everything. Clicking it drops down the switcher: the list of workspaces, a line for
   hidden workspaces (shown while ⌥ is held) and "New workspace". Tabs or dots were
   rejected because they stop working past two workspaces.
2. **Places:** Everything, Up next, Unsorted, Not organized (with a count badge while any
   bookmark is waiting for the AI).
3. **Folders:** each with its own colour and icon. ⌘1–9 jump to the first nine.
   "New folder" at the end.
4. **Footer:** the Knoa wordmark, the AI activity indicator (✦ and the number of
   bookmarks being organized; nothing while the AI is idle) and the settings button.

### Search pill (top centre, floating)

- One field, about 620 × 50 px, centred over the content area.
- Contents: search icon, scope chip (the selected place or folder, e.g. "Reading"),
  input ("Just start typing to search"), live result count, and the **view options**
  button.
- Typing anywhere in the window focuses the field and starts the search. Results update
  on every keystroke and never wait for the network.
- Focusing the empty field shows recent searches and the main shortcuts.
- Below the pill: **kind chips** (All, Movies, Articles, Videos, Code, Recipes, Products,
  Books). Only kinds with results in the current view are shown. Chips filter instantly.
- The typed filter syntax and chips from the research report (`kind:`, `site:`, `#tag`,
  field comparisons) live in this field.

### Content area

- **Up next strip** at the top of Everything: a single row of compact items the user
  marked to read or watch, with a count and a hide (✕) button. It is shown by default.
  Once hidden it stays hidden until turned back on in view options; the Up next place in
  the sidebar is always there.
- **Date groups:** Today, Yesterday, This week, Last week, then months, then years. Each
  group has a small label and its cards below.
- **Masonry cards** (default layout). Grid and list are available in view options.
- Three columns while the detail panel is open, four when it is closed.

### Cards

- Image first, at its natural ratio: posters tall, videos 16:9, articles and recipes
  landscape, products square.
- A small kind icon on the image; an "Up next" badge when queued; a play button on
  videos.
- Below the image: title, favicon + site + one key fact (year and runtime, author,
  language), and a three-line summary.
- Repositories are text-only dark cards (owner/name in monospace, summary).
- Bookmarks the AI has not processed yet are compact dashed cards with the favicon,
  title, site and a "Not organized yet" or "Organizing…" chip. They must look good:
  after the first import, most of the library looks like this.
- Hover lifts a card by 2 px. **The selected card has no styling at all**: no outline,
  ring, glow, dimming or marker. The detail panel shows what is selected.

### Date rail (right edge of the content)

- A column of short horizontal lines, one per date group, like the message markers in AI
  chat apps.
- Line length reflects how many bookmarks the period has; the current period is the
  strongest.
- Hovering a line widens it and shows its label and count ("October · 6"). Clicking
  jumps there.

### Detail panel (right, floating, ~340 px)

- Image, kind and site, title (large serif), AI summary marked with ✦, typed fields in a
  two-column grid, folders the AI added marked with ✦ and an Undo.
- For unprocessed bookmarks: "Only the title and link so far" and an Organize button.
- Actions at the bottom: **Open** (↵), Up next toggle, Copy link, Delete.
- In Trash: how long until it is deleted for good, with **Restore** and Delete now.
- A close button; closing gives the cards the full width.

### Popovers

- **View options** (button in the search pill): layout (Masonry, Grid, List), card size,
  show summaries, group by date, sort, show the Up next strip.
- **Settings** (gear in the sidebar footer): appearance (theme, wallpaper, wallpaper blur,
  panel opacity) and a link to all settings.
- **Workspace switcher** (workspace button in the titlebar), dropping down from it.

## Keyboard

| Key | Action |
| --- | --- |
| Any letter | Search |
| Arrows | Move through cards |
| ↵ | Open in the browser |
| Space | Show or hide the detail panel |
| L | Add to or remove from Up next |
| ⌘C | Copy link |
| ⌫ | Move to Trash |
| ⌘1–9 | Go to folder |
| ⌘K | Command palette |
| ⌘, | Settings |
| Esc | Clear the search, then close the detail panel |

## Look and themes

- **Themes, not just light and dark.** The theme system supports any number of themes.
  The default is a calm, solid light theme.
- **Wallpaper is optional.** With a wallpaper, the sidebar, search pill and detail panel
  become translucent "glass" panels; the user can blur the wallpaper and set panel
  opacity. Without one, the same panels are solid. Both must look finished.
- Text on the wallpaper itself (group labels) sits on a small glass chip so it stays
  readable on any image.
- AI-made values use one accent (✦), separate from the theme's accent.

## Up next

A user-managed queue of bookmarks to read or watch:

- Added and removed with L, the bookmark button in the detail panel, or the context menu.
- Shown as the strip on Everything, as a place in the sidebar and as a badge on cards.
- Set only by the user, never by the AI.

## Rejected during exploration

| Idea | Why not |
| --- | --- |
| Search-first home (empty screen with one big search field) | Adds a step before the list; slower for the most common task |
| Workspace tabs or dots | Do not scale past two workspaces |
| Search in the sidebar or a toolbar | Weaker than a centred pill; the pill is the main control |
| Icon-only sidebar | Folders need their names |
| Folder "piles", timeline-only and split reader layouts | Interesting, but less familiar and slower to scan |
| Any styling on the selected card (outline, glow, dimming, marker) | Looks bad, especially on wallpaper; the detail panel is enough |

## Implementation notes

See [ADR 0007](../adr/0007-main-screen-rendering.md) for masonry, glass panels and the
date rail in GPUI.

## Decided after approval

- Workspaces shipped without hidden workspaces. The switcher lists them with the
  browsers each syncs; the pencil opens the workspace editor. "Set aside" shows in
  places only while the workspace's guideline has set something aside.
- Trash shows in places only while it holds something, with an Empty Trash strip above
  its cards ([ADR 0012](../adr/0012-deleting-bookmarks-through-trash.md)).
