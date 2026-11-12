#!/usr/bin/env bash
# Builds target/release/Knoa.app as Knoa Dev unless KNOA_CHANNEL=release.
#
#   KNOA_CHANNEL=release  distribution identity, real library and Sparkle (package.sh).
#   KNOA_DEV=1           debug profile in target/debug/Knoa.app instead of release.
#   KNOA_SIGN_IDENTITY  codesign identity, like "Developer ID Application". Default: ad hoc.
#   KNOA_UNIVERSAL=1    build for Apple silicon and Intel, not just this Mac.
#   KNOA_BUILD          CFBundleVersion. Default: the number of commits on this branch.
set -euo pipefail
cd "$(dirname "$0")/.."

DEV=${KNOA_DEV:-0}
if [[ $DEV == 1 ]]; then
    APP=target/debug/Knoa.app
else
    APP=target/release/Knoa.app
fi
BUNDLE=crates/knoa/bundle
IDENTITY=${KNOA_SIGN_IDENTITY:--}
VERSION=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -1)
BUILD=${KNOA_BUILD:-$(git rev-list --first-parent --count HEAD 2>/dev/null || echo 1)}

# actool ships with Xcode, not with the Command Line Tools. Only actool gets Xcode's
# DEVELOPER_DIR: build scripts track it, so exporting it would make cargo rebuild crates
# whenever builds switch between make and plain cargo.
actool() {
    if xcrun --find actool >/dev/null 2>&1; then
        xcrun actool "$@"
    else
        DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcrun actool "$@"
    fi
}
actool --version >/dev/null 2>&1 || { echo "bundle.sh: actool not found; install Xcode 26 or later" >&2; exit 1; }

build_binary() {
    if [[ $DEV == 1 ]]; then
        cargo build
        cp target/debug/knoa "$1"
    elif [[ ${KNOA_UNIVERSAL:-0} == 1 ]]; then
        cargo build --release --locked --target aarch64-apple-darwin
        cargo build --release --locked --target x86_64-apple-darwin
        lipo -create -output "$1" \
            target/aarch64-apple-darwin/release/knoa target/x86_64-apple-darwin/release/knoa
    else
        cargo build --release --locked
        cp target/release/knoa "$1"
    fi
}

# Knoa Dev is an app of its own, so it runs beside an installed Knoa and tools that find
# apps by bundle ID reach the build being worked on.
write_info_plist() {
    local plist=$APP/Contents/Info.plist
    sed -e "s/@VERSION@/$VERSION/" -e "s/@BUILD@/$BUILD/" "$BUNDLE/Info.plist" > "$plist"
    if [[ ${KNOA_CHANNEL:-} != release ]]; then
        plutil -replace CFBundleIdentifier -string dev.knoa.Knoa-Dev "$plist"
        plutil -replace CFBundleName -string "Knoa Dev" "$plist"
        plutil -replace CFBundleDisplayName -string "Knoa Dev" "$plist"
        for key in SUFeedURL SUPublicEDKey SUEnableAutomaticChecks SURequireSignedFeed SUVerifyUpdateBeforeExtraction; do
            plutil -remove "$key" "$plist"
        done
    fi
}

# AppIcon.icon is the Icon Composer source. actool turns it into Assets.car, which macOS 26+
# renders in every icon appearance, and AppIcon.icns for older systems. Info.plist already
# names both, so actool's partial plist is thrown away.
compile_icon() {
    local partial_plist
    partial_plist=$(mktemp)
    actool "$BUNDLE/AppIcon.icon" --compile "$APP/Contents/Resources" --app-icon AppIcon \
        --output-partial-info-plist "$partial_plist" --platform macosx --minimum-deployment-target 11.0 \
        --output-format human-readable-text --errors --warnings >/dev/null
    rm -f "$partial_plist"
}

# Knoa isn't sandboxed, so Sparkle's XPC services aren't needed. Headers and modules are
# only for compiling against it.
embed_sparkle() {
    local sparkle framework
    sparkle=$(scripts/sparkle.sh)
    framework=$APP/Contents/Frameworks/Sparkle.framework
    mkdir -p "$APP/Contents/Frameworks"
    ditto "$sparkle/Sparkle.framework" "$framework"
    rm -rf "$framework"/{XPCServices,Headers,PrivateHeaders,Modules} \
        "$framework"/Versions/B/{XPCServices,Headers,PrivateHeaders,Modules}
}

# Sign nested code first, never with --deep: each piece needs its own signature. A
# Developer ID build gets the hardened runtime and a secure timestamp, which notarization
# requires. An ad hoc build gets neither: library validation would refuse an ad hoc
# Sparkle, and there's no timestamp server for ad hoc signatures.
sign() {
    local flags=(--force --sign "$IDENTITY")
    if [[ $IDENTITY != - ]]; then
        flags+=(--options runtime --timestamp)
    fi
    local framework=$APP/Contents/Frameworks/Sparkle.framework
    if [[ -d $framework ]]; then
        codesign "${flags[@]}" "$framework/Versions/B/Autoupdate"
        codesign "${flags[@]}" "$framework/Versions/B/Updater.app"
        codesign "${flags[@]}" "$framework"
    fi
    codesign "${flags[@]}" "$APP"
}

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
build_binary "$APP/Contents/MacOS/Knoa"
write_info_plist
compile_icon
if [[ ${KNOA_CHANNEL:-} == release ]]; then
    embed_sparkle
fi
sign
codesign --verify --strict --deep "$APP"
echo "Built $APP ($VERSION, build $BUILD, $(lipo -archs "$APP/Contents/MacOS/Knoa"), signed by $IDENTITY)"
