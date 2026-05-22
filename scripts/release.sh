#!/usr/bin/env bash
# release.sh — cut a blog-toolkit release
#
# After pushing the tag, GitHub Actions (release.yml) automatically:
#   1. Builds macOS arm64, Linux x86_64, Windows installer
#   2. Creates the GitHub Release with all artifacts
#
# Usage:
#   ./scripts/release.sh            # auto-bump patch (0.1.0 → 0.1.1)
#   ./scripts/release.sh 1.0.0      # explicit version
#   ./scripts/release.sh --minor    # bump minor  (0.1.0 → 0.2.0)
#   ./scripts/release.sh --major    # bump major  (0.1.0 → 1.0.0)
#   ./scripts/release.sh --dry-run  # show plan, don't execute

set -euo pipefail

CARGO="Cargo.toml"
DRY_RUN=false

CURRENT=$(grep '^version' "$CARGO" | head -1 | sed 's/version = "\(.*\)"/\1/')
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT"

# ── Compute target version ────────────────────────────────────────────────────
case "${1:-}" in
    --dry-run)             DRY_RUN=true; NEW="$MAJOR.$MINOR.$((PATCH + 1))" ;;
    --patch|"")            NEW="$MAJOR.$MINOR.$((PATCH + 1))" ;;
    --minor)               NEW="$MAJOR.$((MINOR + 1)).0" ;;
    --major)               NEW="$((MAJOR + 1)).0.0" ;;
    [0-9]*.[0-9]*.[0-9]*) NEW="$1" ;;
    *)
        echo "Usage: $0 [--patch|--minor|--major|--dry-run|<version>]"
        exit 1 ;;
esac

TAG="v$NEW"

# ── Pre-flight checks ─────────────────────────────────────────────────────────
ERRORS=()

[[ "$NEW" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || ERRORS+=("Version must be semver, got: $NEW")
git rev-parse "$TAG" &>/dev/null && ERRORS+=("Tag $TAG already exists")
[[ -n "$(git status --porcelain)" ]] && ERRORS+=("Working tree is dirty — commit or stash first")

if [[ ${#ERRORS[@]} -gt 0 ]]; then
    for e in "${ERRORS[@]}"; do echo "❌ $e"; done
    exit 1
fi

# ── Show plan ─────────────────────────────────────────────────────────────────
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Release plan — blog-toolkit"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Cargo.toml : $CURRENT → $NEW"
echo "  Git tag    : $TAG"
echo "  Branch     : $(git branch --show-current)"
echo ""

LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
if [[ -n "$LAST_TAG" ]]; then
    COUNT=$(git log "$LAST_TAG"..HEAD --oneline | wc -l | tr -d ' ')
    echo "  Commits since $LAST_TAG: $COUNT"
    git log "$LAST_TAG"..HEAD --oneline --no-decorate | sed 's/^/    /'
else
    echo "  (no previous tag — first release)"
fi
echo ""

$DRY_RUN && { echo "Dry run — nothing done."; exit 0; }

read -r -p "Proceed? [y/N] " CONFIRM
[[ "$CONFIRM" =~ ^[Yy]$ ]] || { echo "Aborted."; exit 0; }

# ── Bump + commit + tag ───────────────────────────────────────────────────────
if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "s/^version = \"$CURRENT\"/version = \"$NEW\"/" "$CARGO"
else
    sed -i    "s/^version = \"$CURRENT\"/version = \"$NEW\"/" "$CARGO"
fi

git add "$CARGO"
git commit -m "chore: release $TAG"
git tag "$TAG"

git push origin HEAD
git push origin "$TAG"

echo ""
echo "🚀  $TAG pushed — CI is building:"
echo "    https://github.com/$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/' | sed 's/.*github.com[:/]//')/actions"
