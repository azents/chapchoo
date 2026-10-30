#!/bin/sh
# Create once through the Kubernetes client; never print or replace the proof.
set -eu
umask 077
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT HUP INT TERM
: "${POD_NAMESPACE:?Namespace is required}"
: "${SETUP_SECRET_NAME:?Setup Secret name is required}"
: "${SETUP_SECRET_KEY:?Setup Secret field is required}"

read_existing() {
  kubectl --namespace "$POD_NAMESPACE" get secret "$SETUP_SECRET_NAME" \
    --ignore-not-found -o "jsonpath={.data['$SETUP_SECRET_KEY']}" > "$work/encoded" 2>"$work/read-error"
}

validate_existing() {
  # Empty/missing fields in an existing Secret are invalid, never overwritten.
  base64 --decode "$work/encoded" > "$work/existing" 2>/dev/null || return 1
  [ "$(wc -c < "$work/existing")" -eq 43 ] || return 1
  LC_ALL=C grep -Eq '^[A-Za-z0-9_-]{42}[AEIMQUYcgkosw048]$' "$work/existing"
}

# Fetch the named object separately from its field so a malformed existing
# Secret is distinguishable from an absent object. No list permission needed.
kubectl --namespace "$POD_NAMESPACE" get secret "$SETUP_SECRET_NAME" \
  --ignore-not-found -o name > "$work/name" 2>"$work/read-error" || {
  echo 'Could not inspect the named setup Secret' >&2; exit 1;
}
if [ -s "$work/name" ]; then
  read_existing
  validate_existing || { echo 'Existing setup proof is invalid' >&2; exit 1; }
  echo 'Existing setup proof retained'
  exit 0
fi
head -c 32 /dev/urandom > "$work/random"
[ "$(wc -c < "$work/random")" -eq 32 ]
base64 "$work/random" | tr '+/' '-_' | tr -d '=\n' > "$work/value"
if kubectl --namespace "$POD_NAMESPACE" create secret generic "$SETUP_SECRET_NAME" \
  --from-file="$SETUP_SECRET_KEY=$work/value" >/dev/null 2>"$work/create-error"; then
  echo 'Initial setup proof created'
else
  # Another installer may have won. Verify that object; never replay a create
  # or overwrite it, and do not expose API error payloads or secret values.
  read_existing
  validate_existing || { echo 'Setup proof creation or concurrent validation failed' >&2; exit 1; }
  echo 'Concurrent setup proof retained'
fi
