# 0004: gpui-component for primitives, own design on top

- Status: Accepted
- Date: 2026-11-04

## Context

Design quality matters from the first milestone. Options:

1. **Build every component on raw GPUI.** Full control, but text input (IME, selection)
   and virtualized lists alone take weeks.
2. **Use [gpui-component](https://github.com/longbridge/gpui-component) as-is.** Fast, but
   the app looks like every other gpui-component app.
3. **Combine the two.**

## Decision

Option 3:

- **gpui-component** provides the complex primitives: text input, virtualized list,
  scrollbars, popovers and menus.
- **Knoa** provides the theme (colors, type scale, spacing, radii, animation) applied to
  those primitives, and its own cards, search bar, detail view, command palette and
  animations.
- If a gpui-component piece gets in the way of the design, we replace that piece with our
  own.

## Consequences

- Fast start without giving up the look.
- Our `gpui-pre` version must match the one gpui-component pins
  ([ADR 0001](0001-gpui-via-gpui-pre-snapshots.md)), so GPUI upgrades follow its releases.
- The theme must cover both our components and gpui-component's.
