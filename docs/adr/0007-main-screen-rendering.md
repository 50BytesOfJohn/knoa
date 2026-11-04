# 0007: Rendering the main screen in GPUI

- Status: Accepted
- Date: 2026-11-04

## Context

The approved main screen ([design](../design/main-screen.md)) needs three things GPUI
does not provide out of the box:

1. **Masonry cards** with varying heights in a library of thousands of bookmarks.
2. **Glass panels** (translucent, blurred) over a user-chosen wallpaper.
3. **A date rail** that jumps to a date group in a long scrolling list.

What GPUI 0.3.11 offers (checked in the `gpui-pre` source):

- `uniform_list` and `list` virtualize rows; neither lays out masonry columns.
- `WindowBackgroundAppearance::Blurred` blurs the desktop behind the whole window
  (an `NSVisualEffectView`). There is no way to blur one element over another
  (no backdrop filter); blur exists only on shadows.
- Apple's Liquid Glass material is not available.

## Decision

- **Masonry:** our own virtualized element. Card heights are computed from data we already
  have (image aspect ratio, title and summary line counts at the current width), so the
  layout of every card is known without rendering it. Cards are placed into the shortest
  column; only cards that intersect the viewport are rendered. Layouts are cached per
  width and recomputed when the width, filter or card size changes. Grid and list views
  use GPUI's `uniform_list`.
- **Glass panels:** when a wallpaper is set, Knoa decodes it once, makes a blurred copy on
  the background executor (at the chosen blur radius) and caches it. Each floating panel
  draws the part of the blurred copy that lies under it, clipped to its rounded rect,
  then a translucent fill, a hairline highlight and a shadow. The wallpaper is static,
  so this matches a real backdrop blur.
- **Cards under the search pill** are not blurred (that would need a real backdrop
  filter). The pill is drawn nearly opaque.
- **Date rail:** the masonry layout records the y offset of each date group's header;
  clicking a rail line scrolls to that offset. The rail reads the same group list as the
  content, so it needs no extra queries.
- Desktop blur (`WindowBackgroundAppearance::Blurred`) is not used for the alpha.

## Consequences

- Masonry is our code to maintain and test (column placement, virtualization, scroll
  anchoring when cards change height after the AI fills them in).
- Card height must be computable without rendering. Card designs keep to a fixed set of
  regions (image, title, meta, summary) with known sizes.
- Changing the wallpaper or blur radius recomputes one blurred image, off the main
  thread.
- If GPUI gains a backdrop filter later, the pre-blurred copy can be replaced without
  changing the panel design.
