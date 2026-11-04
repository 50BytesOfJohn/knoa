# Contributing to Knoa

Thanks for helping. Knoa is free software, and contributions of every size are welcome:
bug reports, ideas, docs, code.

## Before you start

- For anything bigger than a small fix, open an issue first so we can agree on the approach.
- Read [docs/VISION.md](docs/VISION.md) (what Knoa should feel like) and
  [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) (how it's built).

## Making a change

1. Fork and branch from `main`.
2. `make dev` runs the app; `make fmt` formats.
3. `make check` must pass. It's exactly what CI runs.
4. Keep pull requests focused, and update docs when behavior or design changes.

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
