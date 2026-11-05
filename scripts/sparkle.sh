#!/usr/bin/env bash
# Downloads the pinned Sparkle release once, checks its hash, and prints the directory it's
# unpacked in: Sparkle.framework, and the release tools in bin/.
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=2.10.1
SHA256=1a856d2a9edb8e628de88121d7be04cf0e2167f8641650b78fca00b14244cf27
DIR=target/sparkle/$VERSION

if [[ ! -d $DIR/Sparkle.framework ]]; then
    archive=$(mktemp)
    curl -fsSL -o "$archive" \
        "https://github.com/sparkle-project/Sparkle/releases/download/$VERSION/Sparkle-$VERSION.tar.xz"
    echo "$SHA256  $archive" | shasum -a 256 -c --quiet
    rm -rf "$DIR"
    mkdir -p "$DIR"
    tar -xf "$archive" -C "$DIR"
    rm -f "$archive"
fi
echo "$DIR"
