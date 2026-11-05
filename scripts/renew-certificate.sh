#!/usr/bin/env bash
# Renews the Developer ID Application certificate, on the maintainer's Mac. It makes a new
# private key and a certificate signing request, waits while you get the certificate from
# Apple's developer site, installs both in the login keychain, checks that they sign, and
# updates the certificate the Release workflow uses. The old certificate stays until it
# expires; signing picks the one that expires last (scripts/identity.sh). No sudo needed.
set -euo pipefail
cd "$(dirname "$0")/.."

PAGE=https://developer.apple.com/account/resources/certificates/add
CSR=$HOME/Downloads/Knoa.certSigningRequest

bold() { printf '\n\033[1m%s\033[0m\n' "$*"; }
note() { printf '   %s\n' "$*"; }
fail() { printf '\n\033[31m%s\033[0m\n' "$*" >&2; exit 1; }

WORK=$(mktemp -d)
trap 'rm -rf -- "${WORK:?}"' EXIT
chmod 700 "$WORK"

make_request() {
    bold "1/4  Signing request"
    openssl genrsa -out "$WORK/key.pem" 2048 2>/dev/null
    openssl req -new -key "$WORK/key.pem" -subj "/CN=Knoa Developer ID" -out "$CSR"
    note "Made a new private key, and ~/Downloads/$(basename "$CSR") for Apple."
}

# Apple doesn't let API keys create Developer ID certificates, so this step is in the
# browser.
wait_for_certificate() {
    bold "2/4  Certificate from Apple"
    note "Opening developer.apple.com. There:"
    note "  1. Pick \"Developer ID Application\", then Continue."
    note "  2. Pick \"G2 Sub-CA (Xcode 11.4.1 or later)\"."
    note "  3. Choose File: ~/Downloads/$(basename "$CSR") (shown in Finder), then Continue."
    note "  4. Click Download."
    note "Waiting for developerID_application.cer in Downloads…"
    touch "$WORK/started"
    open "$PAGE"
    open -R "$CSR"
    until CER=$(find "$HOME/Downloads" -maxdepth 1 -name 'developerID_application*.cer' -newer "$WORK/started" | head -1) &&
        [[ -n $CER ]]; do
        sleep 2
    done
    openssl x509 -inform DER -in "$CER" -out "$WORK/cert.pem" 2>/dev/null ||
        fail "$CER isn't a certificate."
    [[ $(openssl x509 -in "$WORK/cert.pem" -noout -pubkey) == $(openssl pkey -in "$WORK/key.pem" -pubout) ]] ||
        fail "$CER wasn't made from this request. Upload ~/Downloads/$(basename "$CSR") and download again."
    note "Got $(openssl x509 -in "$WORK/cert.pem" -noout -subject | sed 's/.*CN *= *\([^,]*\).*/\1/'),"
    note "valid until $(openssl x509 -in "$WORK/cert.pem" -noout -enddate | sed 's/notAfter=//')."
}

install_identity() {
    bold "3/4  Keychain"
    # codesign needs the intermediate that issued the certificate to build its chain.
    curl -fsSL -o "$WORK/g2.cer" https://www.apple.com/certificateauthority/DeveloperIDG2CA.cer
    security import "$WORK/g2.cer" -k login.keychain-db >/dev/null 2>&1 || true
    local password
    password=$(openssl rand -hex 16)
    openssl pkcs12 -export -in "$WORK/cert.pem" -inkey "$WORK/key.pem" -name "Knoa Developer ID" \
        -certpbe PBE-SHA1-3DES -keypbe PBE-SHA1-3DES -macalg sha1 \
        -passout "pass:$password" -out "$WORK/identity.p12"
    security import "$WORK/identity.p12" -k login.keychain-db -P "$password" \
        -T /usr/bin/codesign -T /usr/bin/security >/dev/null
    check_signing
    rm -f -- "$CSR" "$CER"
    note "Installed in your login keychain and signed a test file with it."
    note "Your Mac's backups now hold the only copy of its private key."
}

check_signing() {
    local sha
    sha=$(openssl x509 -in "$WORK/cert.pem" -noout -fingerprint -sha1 | sed 's/.*=//; s/://g')
    [[ $(scripts/identity.sh) == "$sha" ]] ||
        fail "The keychain doesn't offer the new certificate for signing."
    cp /usr/bin/true "$WORK/signing-test"
    if ! codesign --force --sign "$sha" --options runtime --timestamp "$WORK/signing-test" 2>/dev/null ||
        ! codesign --verify --strict "$WORK/signing-test"; then
        fail "The new certificate is installed, but codesign couldn't sign with it."
    fi
}

make_request
wait_for_certificate
install_identity
bold "4/4  Release workflow"
scripts/setup-release.sh --certificate-only
