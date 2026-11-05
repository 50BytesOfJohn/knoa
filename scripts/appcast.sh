#!/usr/bin/env bash
# Writes target/dist/appcast.xml: the Sparkle feed that announces the release packaged in
# target/dist. Each release's feed lists only itself; Info.plist reads it from the latest
# GitHub release, so publishing a release is what ships the update.
#
#   SPARKLE_PRIVATE_KEY  the EdDSA key that signs the update and the feed
#   RELEASE_NOTES        a Markdown file, shown in Sparkle's update window
set -euo pipefail
cd "$(dirname "$0")/.."

: "${SPARKLE_PRIVATE_KEY:?}"
: "${RELEASE_NOTES:?}"
REPOSITORY=$(sed -n 's/^repository = "\(.*\)"$/\1/p' Cargo.toml | head -1)
VERSION=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -1)
DIST=target/dist
SPARKLE=$(scripts/sparkle.sh)

# generate_appcast reads every archive in a folder and picks up notes named like them.
feed=$(mktemp -d)
cp "$DIST/Knoa-$VERSION.zip" "$feed/"
cp "$RELEASE_NOTES" "$feed/Knoa-$VERSION.md"
echo "$SPARKLE_PRIVATE_KEY" | "$SPARKLE/bin/generate_appcast" "$feed" \
    --ed-key-file - \
    --download-url-prefix "$REPOSITORY/releases/download/v$VERSION/" \
    --embed-release-notes \
    --full-release-notes-url "$REPOSITORY/blob/main/CHANGELOG.md" \
    --link "$REPOSITORY" \
    --maximum-deltas 0 \
    -o "$DIST/appcast.xml"
rm -rf "$feed"
echo "Wrote $DIST/appcast.xml"
