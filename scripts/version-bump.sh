#!/usr/bin/env bash
# Cyrius version bump. `cyrius.cyml` reads version from `${file:VERSION}`, so the
# VERSION file is the single source of truth; this writes it and re-stamps the
# dist bundle. (The Rust-era bump — which sed'd Cargo.toml + ran cargo — is
# retired; the Rust project now lives frozen under rust-old/.)
set -euo pipefail
[ $# -ne 1 ] && echo "Usage: $0 <version>" && exit 1
NEW_VERSION="$1"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
echo "$NEW_VERSION" > "$REPO_ROOT/VERSION"
cd "$REPO_ROOT"
cyrius distlib >/dev/null 2>&1 || true   # re-stamp dist/garjan.cyr with the new version
echo "Bumped to ${NEW_VERSION} (VERSION + dist re-stamped). Update CHANGELOG, then tag."
