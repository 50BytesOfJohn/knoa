# Contributing to Knoa

Thanks for helping. Knoa is free software, and contributions of every size are welcome:
bug reports, ideas, docs, code.

## Before you start

- For anything bigger than a small fix, open an issue first so we can agree on the approach.
  Ideas and questions go to [Discussions](https://github.com/50BytesOfJohn/knoa/discussions).
- Everyone follows the [Code of Conduct](CODE_OF_CONDUCT.md). Security issues are reported
  privately, as [SECURITY.md](SECURITY.md) describes.
- Read [docs/VISION.md](docs/VISION.md) (what Knoa should feel like) and
  [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) (how it's built).

## Making a change

1. Fork and branch from `main`.
2. `make dev` runs the app; `make fmt` formats.
3. `make check` must pass. It's exactly what CI runs.

   `make` builds at background priority, one job per efficiency core, so your Mac stays
   cool and quiet. Builds take longer this way. To change that on your machine, put any
   of these in a `local.mk` file at the repo root (git ignores it), or pass them to a
   single run, like `make check FAST=1`:

   ```make
   # Build at full speed
   FAST = 1
   # Or keep background priority with a different job count
   CARGO_BUILD_JOBS = 6
   ```
4. Keep pull requests focused, and update docs when behavior or design changes.
5. Title commits and pull requests as a plain sentence that starts with a verb: "Add Arc
   import", "Fix search for quoted words". Titles become the changelog; "Add", "Fix" and
   "Remove" get their own sections. Put `[skip changelog]` in the message of a change
   users won't notice.

## License of contributions

Knoa is licensed under the [GNU GPL v3.0 or later](LICENSE). Contributions are accepted under
the same license ("inbound = outbound"); there is no contributor license agreement.

To certify that you wrote the change or otherwise have the right to submit it, sign off
each commit under the [Developer Certificate of Origin](https://developercertificate.org):

```sh
git commit -s
```

This adds a `Signed-off-by: Your Name <you@example.com>` line to the commit message.

## The Knoa name

The code is free; the name and logo identify the official project. You can fork and
redistribute Knoa under the GPL, but a modified or redistributed build must use a different
name and icon. See [TRADEMARK.md](TRADEMARK.md).
