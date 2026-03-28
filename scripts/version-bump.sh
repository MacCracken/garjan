#!/usr/bin/env bash
set -euo pipefail
[ $# -ne 1 ] && echo "Usage: $0 <version>" && exit 1
NEW_VERSION="$1"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
echo "$NEW_VERSION" > "$REPO_ROOT/VERSION"
sed -i "s/^version = \".*\"/version = \"${NEW_VERSION}\"/" "$REPO_ROOT/Cargo.toml"
cd "$REPO_ROOT" && cargo generate-lockfile 2>/dev/null
echo "Bumped to ${NEW_VERSION}. Tag and push."
