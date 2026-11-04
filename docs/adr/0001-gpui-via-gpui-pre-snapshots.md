# 0001: Use GPUI via the `gpui-pre` snapshot crates

- Status: Accepted
- Date: 2026-11-04

## Context

We want GPUI (Zed's UI framework) for a fast, native-feeling desktop app. GPUI is pre-1.0
and developed inside the Zed monorepo. As of November 2026 there are three ways to depend on
it:

1. **`gpui` on crates.io**, last published as 0.2.2 in October 2025. It's over a year behind
   Zed and uses an older API (`Application::new()` instead of `gpui_platform::application()`).
2. **A git dependency on `zed-industries/zed`** pinned to a commit. Always current, but every
   fresh checkout and CI run clones a very large repository, and pins are bare commit hashes.
3. **`gpui-pre` / `gpui-pre-platform` on crates.io**: weekly snapshots of Zed's GPUI crates
   (0.3.11 = zed@f816e1f, published 2026-11-02), maintained by the gpui-component authors.
   gpui-component 0.9, the main GPUI component library, depends on exactly these snapshots.

## Decision

Use option 3. In `[workspace.dependencies]`, rename the snapshot packages to the names the
code expects:

```toml
gpui = { package = "gpui-pre", version = "=0.3.11" }
gpui_platform = { package = "gpui-pre-platform", version = "=0.3.11", features = ["font-kit", "runtime_shaders"] }
```

- **Exact pins** (`=`), because any snapshot may change GPUI's API. All snapshot crates are
  bumped together, and kept equal to the version gpui-component pins once we adopt it.
- **`font-kit`** is required on macOS for glyph rendering.
- **`runtime_shaders`** compiles Metal shaders at startup instead of build time, so building
  doesn't depend on the full Xcode Metal toolchain (`xcrun metal`), which isn't present with
  Command Line Tools alone or on every CI image. Revisit for release builds, where
  precompiled shaders shave startup time.

## Consequences

- Recent GPUI API, crates.io-speed builds, and a free upgrade path to gpui-component.
- We depend on a third party republishing Zed's code. If that stops, switching to a git pin
  of Zed (option 2) is a `Cargo.toml` change; the crate names in code stay the same.
- Upgrades are deliberate: bump the pins, fix API breaks, run `make check`.
