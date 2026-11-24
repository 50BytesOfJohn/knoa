# Research: the top bar

- Date: 2026-11-24
- Question: what do people expect from the search area of a library app like Knoa? Where
  do view controls go, how should type filters, suggestions and applied filters work, so
  the bar feels familiar from the first use?
- Method: desk research (Apple's guidelines, Baymard Institute's search and filter
  studies, the docs of Gmail and Raycast) plus a review of Finder, Photos, Mail, Gmail,
  Raycast, Raindrop.io, mymind, Linear and Notion. No user testing yet.
- Builds on [main-screen research](main-screen.md) and round 1 of the search
  explorations ([search-explorations.html](../design/search-explorations.html)).
- Outcome: [top bar design](../design/top-bar.md). The options compared are drawn in
  [top-bar-explorations.html](../design/top-bar-explorations.html).

## Findings

1. **View controls belong at the top right.** Apple's toolbar guidelines give the
   trailing edge to controls that must stay reachable: view switchers, inspectors, the
   More menu. Finder, Photos, Mail and Notes all put "how it looks" there.
2. **Keep "what it shows" apart from "how it looks".** Linear and Notion keep filters with
   the search and put layout, sort and density in a separate Display menu. Knoa's sliders
   button inside the pill mixes the two, and looks like a filter button.
3. **Suggestions teach the vocabulary.** Autocomplete helps most when it shows what you
   can search by, not only when it finishes words. Keep it short (about 8–10 rows on
   desktop), visually plain, and set the predicted part apart from what was typed
   (Baymard, via Lucidworks' summary).
4. **Applied filters must be visible and removable.** Without an overview, people can't
   tell why results are missing. Baymard found 28% of large sites still lack one. Each
   filter needs its own remove control, and there should be a way to clear them all.
5. **Refinements can appear when you search.** Gmail shows its filter chips only after a
   search, under the field. No clutter at rest, and a nudge exactly when narrowing.
6. **Browsing still beats searching.** People reach saved things by navigating 56–68% of
   the time and search mostly when they can't remember where something is
   ([main-screen research](main-screen.md), finding 3). One-click kind filters at rest are
   worth keeping.
7. **Only offer what exists.** Raindrop.io lists a filter only when something matches it.
   Counts on every option stop people from narrowing into an empty page.
8. **Hidden filters go unused.** Raycast puts a type dropdown at the end of the search
   field. It's tidy, but a filter behind a closed menu is one most people never open.

## How comparable apps do it

| App | Search | Type / category filters | View controls | What Knoa takes |
| --- | --- | --- | --- | --- |
| Finder | Toolbar field; typing offers "Kinds: Image", "Name matches" | Tokens inside the field | View switcher and group menu in the toolbar | Typed suggestions become tokens; the view switch is its own control |
| Photos | Field at the top; suggestions grouped (People, Places, Things) | Media types in the sidebar | Zoom and filter at the top right | Grouped suggestions; the corner for view |
| Gmail | Wide field; operators as power syntax | Chips under the field after a search | Density in a settings menu | A refinement row that appears when it helps |
| Raycast | One field over a list | Type dropdown at the field's right end | None | Considered for kinds; too hidden |
| Raindrop.io | Field with operators and suggestions | Sidebar filters with counts, only those that match | View menu next to the list | Counts; never offer an empty filter |
| mymind | One calm field; ↵ stacks words | Typing words like "article" | Almost none | Calm at rest; ↵ to narrow |
| Linear | Filter pills ("Label is Bug") | Filter button with field, operator, value | Display menu at the top right | Display menu in the corner; editable tokens |

## What this means for Knoa

- The pill holds only search. View and display controls move to a small capsule at the
  top right of the content; Settings stays in the sidebar footer.
- Kinds stay one click away, in one adaptive row that switches to refinements while
  searching.
- Suggestions are grouped by type, with counts, led by "Search for …" and ending with a few
  bookmarks to open directly.
- The applied filters are the tokens in the pill: one place to read the query, each
  removable, with ⌫ and a Clear action.

## Sources

- [Apple Human Interface Guidelines: Toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars)
- [Apple Human Interface Guidelines: Search fields](https://developer.apple.com/design/human-interface-guidelines/search-fields)
- [Baymard Institute: Autocomplete suggestion design](https://baymard.com/blog/autocomplete-design)
- [Baymard Institute: How to design applied filters](https://baymard.com/blog/how-to-design-applied-filters)
- [Lucidworks: Autocomplete best practices](https://lucidworks.com/blog/best-practices-to-convert-autocomplete)
- [Google Workspace Updates: Gmail search chips](https://workspaceupdates.googleblog.com/2020/02/gmail-search-chips-ga.html)
- [Raycast API: List and the search bar accessory](https://developers.raycast.com/api-reference/user-interface/list)
