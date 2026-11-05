#!/usr/bin/env bash
# Stores what the Release workflow signs with in the repository's `release` environment,
# which only `main` can use. Run it once, and again when a certificate or key changes.
#
#   scripts/release-secrets.sh <DeveloperID.p12> <AuthKey_XXXX.p8> <key id> <issuer id>
#
# The .p12 is the Developer ID Application certificate with its private key, exported from
# Keychain Access; its password is asked for. The .p8 is an App Store Connect API key for
# notarization. The Sparkle key is read from the login keychain.
set -euo pipefail

[[ $# == 4 ]] || { sed -n '5p' "$0" >&2; exit 1; }
P12=$1 P8=$2 KEY_ID=$3 ISSUER_ID=$4
REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner)

gh api -X PUT "repos/$REPO/environments/release" --silent --input - <<'JSON'
{"deployment_branch_policy": {"protected_branches": false, "custom_branch_policies": true}}
JSON
gh api "repos/$REPO/environments/release/deployment-branch-policies" -q '.branch_policies[].name' |
    grep -qx main ||
    gh api -X POST "repos/$REPO/environments/release/deployment-branch-policies" --silent -f name=main -f type=branch

read -rsp "Password of $P12: " P12_PASSWORD
echo
# Keychain Access exports with ciphers OpenSSL 3 only reads with -legacy; LibreSSL has no
# such flag and reads them as is.
certificates() {
    openssl pkcs12 -in "$P12" -passin "pass:$P12_PASSWORD" -nokeys -legacy 2>/dev/null ||
        openssl pkcs12 -in "$P12" -passin "pass:$P12_PASSWORD" -nokeys 2>/dev/null
}
certificates | grep -q "Developer ID Application" ||
    { echo "$P12 has no Developer ID Application certificate, or the password is wrong" >&2; exit 1; }

secret() { gh secret set "$1" --env release --repo "$REPO"; }
base64 -i "$P12" | secret MACOS_CERTIFICATE
printf %s "$P12_PASSWORD" | secret MACOS_CERTIFICATE_PASSWORD
secret NOTARY_KEY < "$P8"
printf %s "$KEY_ID" | secret NOTARY_KEY_ID
printf %s "$ISSUER_ID" | secret NOTARY_ISSUER_ID
security find-generic-password -s https://sparkle-project.org -a ed25519 -w | secret SPARKLE_PRIVATE_KEY
gh secret list --env release --repo "$REPO"
