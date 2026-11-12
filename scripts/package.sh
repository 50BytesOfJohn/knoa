#!/usr/bin/env bash
# Builds the release files in target/dist: Knoa.dmg for people to download, and
# Knoa-<version>.zip for Sparkle. Both hold a universal Knoa.app signed with a Developer ID
# and notarized, with the ticket stapled so Gatekeeper accepts it offline.
#
#   KNOA_SIGN_IDENTITY     a Developer ID Application identity in the keychain.
#   KNOA_CLERK_CLIENT_ID   the production OAuth client id; without it Cloud mode is compiled out.
#   Notary credentials, one of:
#     KNOA_NOTARY_PROFILE                                a `notarytool store-credentials` profile
#     NOTARY_KEY_PATH, NOTARY_KEY_ID, NOTARY_ISSUER_ID   an App Store Connect API key
#
# Needs uv (https://docs.astral.sh/uv), which runs dmgbuild to lay out the DMG window.
set -euo pipefail
cd "$(dirname "$0")/.."

: "${KNOA_SIGN_IDENTITY:?set it to a Developer ID Application identity}"
: "${KNOA_CLERK_CLIENT_ID:?set it to the production Clerk OAuth client id}"
export KNOA_CLERK_CLIENT_ID
if [[ -n ${KNOA_NOTARY_PROFILE:-} ]]; then
    NOTARY=(--keychain-profile "$KNOA_NOTARY_PROFILE")
else
    NOTARY=(--key "${NOTARY_KEY_PATH:?}" --key-id "${NOTARY_KEY_ID:?}" --issuer "${NOTARY_ISSUER_ID:?}")
fi

APP=target/release/Knoa.app
DIST=target/dist
VERSION=$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -1)

KNOA_CHANNEL=release KNOA_DEV=0 KNOA_UNIVERSAL=1 scripts/bundle.sh
rm -rf "$DIST"
mkdir -p "$DIST"

# The DMG opens to the app beside a link to Applications, the usual drag-to-install window.
# scripts/dmg.py lays it out.
make_dmg() {
    uvx --from dmgbuild==1.6.7 dmgbuild -s scripts/dmg.py -D app="$APP" Knoa "$DIST/Knoa.dmg"
    codesign --sign "$KNOA_SIGN_IDENTITY" --timestamp "$DIST/Knoa.dmg"
}

# One submission covers the app too: the ticket lists every signature inside the DMG, so
# both can be stapled.
notarize() {
    local result
    result=$(mktemp)
    xcrun notarytool submit "$DIST/Knoa.dmg" "${NOTARY[@]}" --wait --output-format json > "$result"
    local id status
    id=$(plutil -extract id raw -o - "$result")
    status=$(plutil -extract status raw -o - "$result")
    rm -f "$result"
    if [[ $status != Accepted ]]; then
        echo "package.sh: notarization $id ended as $status" >&2
        xcrun notarytool log "$id" "${NOTARY[@]}" >&2 || true
        exit 1
    fi
    xcrun stapler staple -q "$DIST/Knoa.dmg"
    xcrun stapler staple -q "$APP"
}

make_dmg
notarize
spctl --assess --type execute "$APP"
spctl --assess --type open --context context:primary-signature "$DIST/Knoa.dmg"
ditto -c -k --sequesterRsrc --keepParent "$APP" "$DIST/Knoa-$VERSION.zip"
echo "Packaged $DIST: Knoa.dmg and Knoa-$VERSION.zip ($VERSION)"
