#!/usr/bin/env bash
# Sets up releasing from GitHub, on the maintainer's Mac. It exports the Developer ID
# certificate from the keychain, finds the App Store Connect API key, checks both with
# Apple, and stores them with the Sparkle key as secrets of the repository's `release`
# environment, which only `main` can use. It also saves a local notarytool profile, `knoa`,
# for `make dist`. Run it again when the certificate or a key changes. No sudo needed.
#
#   --certificate-only   update just the certificate, after `make renew-certificate`
set -euo pipefail
cd "$(dirname "$0")/.."

NOTARY_PAGE=https://appstoreconnect.apple.com/access/integrations/api
KEYS_DIR=$HOME/.appstoreconnect/private_keys

bold() { printf '\n\033[1m%s\033[0m\n' "$*"; }
note() { printf '   %s\n' "$*"; }
fail() { printf '\n\033[31m%s\033[0m\n' "$*" >&2; exit 1; }

WORK=$(mktemp -d)
trap 'rm -rf -- "${WORK:?}"' EXIT
chmod 700 "$WORK"

# Keychain exports use ciphers OpenSSL 3 only reads with -legacy; LibreSSL has no such flag.
read_p12() {
    openssl pkcs12 -in "$1" -passin "pass:$2" -nodes -legacy 2>/dev/null ||
        openssl pkcs12 -in "$1" -passin "pass:$2" -nodes 2>/dev/null
}

# Writes each PEM block of $1 to its own file in $2.
split_pem() {
    awk -v dir="$2" '
        /^-----BEGIN / { n++; file = sprintf("%s/%03d.pem", dir, n) }
        file { print > file }
        /^-----END / { close(file); file = "" }
    ' "$1"
}

check_tools() {
    gh auth status >/dev/null 2>&1 || fail "Sign in to GitHub first: gh auth login"
    xcrun notarytool --version >/dev/null 2>&1 || fail "notarytool is missing; install Xcode."
}

# Sets CERT_SHA1 and IDENTITY to the Developer ID Application identity that expires last.
find_identity() {
    local line
    CERT_SHA1=$(scripts/identity.sh 2>/dev/null) ||
        fail "No Developer ID Application certificate in your keychain. Create one with
   make renew-certificate, then run this again."
    line=$(security find-identity -v -p codesigning | grep "$CERT_SHA1")
    IDENTITY=${line#*\"}
    IDENTITY=${IDENTITY%\"}
}

# Exports every identity (macOS has no way to export one from the command line), then
# keeps only the Developer ID certificate and its key, in a .p12 that CI can import.
export_certificate() {
    bold "1/$STEPS  Developer ID certificate"
    find_identity
    note "Found $IDENTITY"
    note "macOS asks for your login password so this script can export it. Click Allow."
    local export_password pem
    export_password=$(openssl rand -hex 16)
    security export -k login.keychain-db -t identities -f pkcs12 \
        -P "$export_password" -o "$WORK/all.p12" >/dev/null ||
        fail "The keychain didn't export the certificate."
    pem=$(read_p12 "$WORK/all.p12" "$export_password") || fail "Couldn't read the exported certificate."
    mkdir "$WORK/blocks"
    split_pem <(printf '%s\n' "$pem") "$WORK/blocks"
    pick_certificate_and_key
    P12_PASSWORD=$(openssl rand -hex 24)
    openssl pkcs12 -export -in "$WORK/cert.pem" -inkey "$WORK/key.pem" -name "$IDENTITY" \
        -certpbe PBE-SHA1-3DES -keypbe PBE-SHA1-3DES -macalg sha1 \
        -passout "pass:$P12_PASSWORD" -out "$WORK/knoa.p12"
    check_importable
    warn_if_expiring
}

pick_certificate_and_key() {
    local block fingerprint public_key
    for block in "$WORK"/blocks/*.pem; do
        fingerprint=$(openssl x509 -in "$block" -noout -fingerprint -sha1 2>/dev/null | sed 's/.*=//; s/://g') || continue
        if [[ $fingerprint == "$CERT_SHA1" ]]; then cp "$block" "$WORK/cert.pem"; fi
    done
    [[ -f $WORK/cert.pem ]] || fail "The export didn't include $IDENTITY."
    public_key=$(openssl x509 -in "$WORK/cert.pem" -noout -pubkey)
    for block in "$WORK"/blocks/*.pem; do
        grep -q "PRIVATE KEY" "$block" || continue
        if [[ $(openssl pkey -in "$block" -pubout 2>/dev/null) == "$public_key" ]]; then
            cp "$block" "$WORK/key.pem"
        fi
    done
    [[ -f $WORK/key.pem ]] || fail "The export didn't include the private key of $IDENTITY."
}

# The same import the Release workflow does, into a throwaway keychain.
check_importable() {
    local keychain=$WORK/check.keychain-db
    security create-keychain -p check "$keychain"
    security import "$WORK/knoa.p12" -k "$keychain" -P "$P12_PASSWORD" >/dev/null
    security find-identity -v -p codesigning "$keychain" | grep -q "$CERT_SHA1" ||
        fail "The exported certificate doesn't import as a signing identity."
    security delete-keychain "$keychain"
    note "Exported, and it imports as a signing identity."
}

warn_if_expiring() {
    local until
    until=$(openssl x509 -in "$WORK/cert.pem" -noout -enddate | sed 's/notAfter=//')
    if ! openssl x509 -in "$WORK/cert.pem" -noout -checkend $((90 * 24 * 3600)) >/dev/null; then
        note "⚠ It expires on $until. Releases can't be signed after that; renew it with"
        note "  make renew-certificate. Apps already shipped keep working."
    else
        note "Valid until $until."
    fi
}

# Sets NOTARY_KEY, NOTARY_KEY_ID and NOTARY_ISSUER_ID.
find_notary_key() {
    bold "2/4  App Store Connect API key (for notarization)"
    NOTARY_KEY=$(find "$HOME/Downloads" "$KEYS_DIR" -maxdepth 1 -name 'AuthKey_*.p8' 2>/dev/null | head -1 || true)
    if [[ -z $NOTARY_KEY ]]; then
        note "Opening App Store Connect. On the Team Keys tab:"
        note "  1. Click + (Generate API Key), name it \"Knoa notarization\", access: Developer."
        note "  2. Click Download next to the new key. Apple lets you download it only once."
        note "Waiting for AuthKey_….p8 in Downloads…"
        open "$NOTARY_PAGE"
        until NOTARY_KEY=$(find "$HOME/Downloads" -maxdepth 1 -name 'AuthKey_*.p8' | head -1) && [[ -n $NOTARY_KEY ]]; do
            sleep 2
        done
    fi
    NOTARY_KEY_ID=$(basename "$NOTARY_KEY" .p8 | sed 's/^AuthKey_//')
    note "Found key $NOTARY_KEY_ID"
    note "Paste the Issuer ID, shown above the keys table on $NOTARY_PAGE"
    read -rp "   Issuer ID: " NOTARY_ISSUER_ID
    NOTARY_ISSUER_ID=$(tr -d '[:space:]' <<<"$NOTARY_ISSUER_ID")
    xcrun notarytool history --key "$NOTARY_KEY" --key-id "$NOTARY_KEY_ID" \
        --issuer "$NOTARY_ISSUER_ID" >/dev/null 2>&1 ||
        fail "Apple didn't accept that key and Issuer ID. Check the Issuer ID and run this again."
    keep_notary_key
}

# Moves the key out of Downloads to where Apple's tools look, and saves a local profile.
keep_notary_key() {
    mkdir -p "$KEYS_DIR"
    chmod 700 "$KEYS_DIR"
    if [[ $(dirname "$NOTARY_KEY") != "$KEYS_DIR" ]]; then
        mv "$NOTARY_KEY" "$KEYS_DIR/"
        NOTARY_KEY=$KEYS_DIR/$(basename "$NOTARY_KEY")
    fi
    chmod 600 "$NOTARY_KEY"
    xcrun notarytool store-credentials knoa --key "$NOTARY_KEY" --key-id "$NOTARY_KEY_ID" \
        --issuer "$NOTARY_ISSUER_ID" >/dev/null
    note "Apple accepted it. Saved to $KEYS_DIR and as the notarytool profile \"knoa\"."
}

find_sparkle_key() {
    bold "3/4  Sparkle update signing key"
    SPARKLE_KEY=$(security find-generic-password -s https://sparkle-project.org -a ed25519 -w 2>/dev/null) ||
        fail "No Sparkle key in your keychain (\"Private key for signing Sparkle updates\")."
    note "Found it in your login keychain. Keep that item backed up: without it, installed"
    note "copies can't verify new updates."
}

store_secrets() {
    bold "4/4  GitHub secrets"
    environment
    base64 -i "$WORK/knoa.p12" | secret MACOS_CERTIFICATE
    printf %s "$P12_PASSWORD" | secret MACOS_CERTIFICATE_PASSWORD
    secret NOTARY_KEY <"$NOTARY_KEY"
    printf %s "$NOTARY_KEY_ID" | secret NOTARY_KEY_ID
    printf %s "$NOTARY_ISSUER_ID" | secret NOTARY_ISSUER_ID
    printf %s "$SPARKLE_KEY" | secret SPARKLE_PRIVATE_KEY
    note "Stored 6 secrets in the \"release\" environment of $REPO (deploys from main only)."
}

store_certificate() {
    bold "2/2  GitHub secrets"
    environment
    base64 -i "$WORK/knoa.p12" | secret MACOS_CERTIFICATE
    printf %s "$P12_PASSWORD" | secret MACOS_CERTIFICATE_PASSWORD
    note "Updated the certificate in the \"release\" environment of $REPO."
}

# The `release` environment, which only `main` can deploy from.
environment() {
    REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner)
    gh api -X PUT "repos/$REPO/environments/release" --silent --input - <<'JSON'
{"deployment_branch_policy": {"protected_branches": false, "custom_branch_policies": true}}
JSON
    gh api "repos/$REPO/environments/release/deployment-branch-policies" -q '.branch_policies[].name' |
        grep -qx main ||
        gh api -X POST "repos/$REPO/environments/release/deployment-branch-policies" --silent \
            -f name=main -f type=branch
}

secret() { gh secret set "$1" --env release --repo "$REPO" >/dev/null; }

check_tools
if [[ ${1:-} == --certificate-only ]]; then
    STEPS=2
    export_certificate
    store_certificate
    bold "Done. The next release is signed with the new certificate."
    exit
fi
STEPS=4
export_certificate
find_notary_key
find_sparkle_key
store_secrets
bold "Done. Release with: make release VERSION=0.1.0"
