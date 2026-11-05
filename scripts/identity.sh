#!/usr/bin/env bash
# Prints the SHA-1 hash of the Developer ID Application identity that expires last. After
# a renewal the old certificate stays in the keychain until it expires, and codesign
# refuses a name that matches two identities, so signing goes by hash.
set -euo pipefail

valid=$(security find-identity -v -p codesigning | awk '/"Developer ID Application/ { print $2 }')
[[ -n $valid ]] || { echo "identity.sh: no Developer ID Application identity in the keychain" >&2; exit 1; }

# The certificate with this SHA-1, as PEM.
certificate() {
    security find-certificate -a -Z -p -c "Developer ID Application" |
        awk -v sha="$1" '/^SHA-1 hash:/ { take = ($3 == sha) } take && !/hash:/'
}

best='' best_end=0
for sha in $valid; do
    end=$(certificate "$sha" | openssl x509 -noout -enddate | sed 's/notAfter=//')
    end=$(date -j -f "%b %e %T %Y %Z" "$end" +%s)
    if ((end > best_end)); then
        best=$sha best_end=$end
    fi
done
echo "$best"
