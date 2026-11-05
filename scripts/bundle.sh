#!/usr/bin/env bash
# Builds target/release/Knoa.app: the release binary, its Info.plist and the app icon.
# Signing is ad hoc; a Developer ID signature and notarization come with the alpha release.
set -euo pipefail
cd "$(dirname "$0")/.."

APP=target/release/Knoa.app
BUNDLE=crates/knoa/bundle
VERSION=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -1)
BUILD=$(git rev-list --first-parent --count HEAD 2>/dev/null || echo 1)

# actool ships with Xcode, not with the Command Line Tools.
if ! xcrun --find actool >/dev/null 2>&1; then
    export DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer
fi
xcrun --find actool >/dev/null || { echo "bundle.sh: actool not found; install Xcode 26 or later" >&2; exit 1; }

cargo build --release --locked

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/knoa "$APP/Contents/MacOS/Knoa"
sed -e "s/@VERSION@/$VERSION/" -e "s/@BUILD@/$BUILD/" "$BUNDLE/Info.plist" > "$APP/Contents/Info.plist"

# AppIcon.icon is the Icon Composer source. actool turns it into Assets.car, which macOS 26+
# renders in every icon appearance, and AppIcon.icns for older systems. Info.plist already
# names both, so actool's partial plist is thrown away.
PARTIAL_PLIST=$(mktemp)
xcrun actool "$BUNDLE/AppIcon.icon" --compile "$APP/Contents/Resources" --app-icon AppIcon \
    --output-partial-info-plist "$PARTIAL_PLIST" --platform macosx --minimum-deployment-target 11.0 \
    --output-format human-readable-text --errors --warnings >/dev/null
rm -f "$PARTIAL_PLIST"

codesign --force --sign - "$APP"
echo "Built $APP ($VERSION, build $BUILD)"
