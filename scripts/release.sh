#!/usr/bin/env sh
# Release the `ops-lib` crate.
#
#   scripts/release.sh --dry-run   preflight + cargo publish --dry-run (default)
#   scripts/release.sh --publish   preflight + publish + tag ops-lib/v<version>
#
# Idempotent: an already published version or an existing tag is skipped, not
# failed. Unless this is a dry run, ops/lib/CHANGELOG.md must carry the released
# version as its latest entry in the `## [x.y.z] - YYYY-MM-DD` format.
set -eu

dry_run=true
for arg in "$@"; do
    case "$arg" in
        --publish) dry_run=false ;;
        --dry-run) dry_run=true ;;
        *)
            echo "unknown argument: $arg" >&2
            exit 2
            ;;
    esac
done

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"

CARGO_TOML="ops/lib/Cargo.toml"
CHANGELOG="ops/lib/CHANGELOG.md"

# --- crate name and version, read from the manifest ------------------------
NAME=$(awk '
    /^\[package\]/ { in_pkg = 1; next }
    /^\[/ { in_pkg = 0 }
    in_pkg && $1 == "name" { gsub(/[",]/, "", $3); print $3; exit }
' "$CARGO_TOML")
VERSION=$(awk '
    /^\[package\]/ { in_pkg = 1; next }
    /^\[/ { in_pkg = 0 }
    in_pkg && $1 == "version" { gsub(/[",]/, "", $3); print $3; exit }
' "$CARGO_TOML")
OUR_REPO=$(sed -n 's/^repository *= *"\(.*\)"/\1/p' "$CARGO_TOML" | head -n1)

if [ -z "$NAME" ] || [ -z "$VERSION" ]; then
    echo "could not read the name and version from $CARGO_TOML" >&2
    exit 1
fi

TAG="ops-lib/v$VERSION"
echo "crate:    $NAME"
echo "version:  $VERSION"
echo "tag:      $TAG"
echo "dry run:  $dry_run"

if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
    {
        echo "## Release $NAME $VERSION"
        echo
        echo "- tag: \`$TAG\`"
        echo "- dry run: \`$dry_run\`"
    } >> "$GITHUB_STEP_SUMMARY"
fi

# --- semver -----------------------------------------------------------------
if ! printf '%s' "$VERSION" | grep -Eq '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$'; then
    echo "version $VERSION is not valid semver" >&2
    exit 1
fi

# --- changelog, skipped on dry runs -----------------------------------------
if [ "$dry_run" = "false" ]; then
    CHANGELOG_LINE=$(grep -m1 -E '^## \[[0-9]+\.[0-9]+\.[0-9]+' "$CHANGELOG" || true)
    CHANGELOG_VERSION=$(printf '%s' "$CHANGELOG_LINE" | sed -E 's/^## \[([^]]+)\].*/\1/')
    VERSION_RE=$(printf '%s' "$VERSION" | sed 's/[][\.^$*+?(){}|]/\\&/g')

    if [ "$CHANGELOG_VERSION" != "$VERSION" ]; then
        echo "$CHANGELOG's latest version is '$CHANGELOG_VERSION', not '$VERSION'" >&2
        exit 1
    fi
    if ! printf '%s' "$CHANGELOG_LINE" | grep -Eq "^## \[$VERSION_RE\] - [0-9]{4}-[0-9]{2}-[0-9]{2}$"; then
        echo "$CHANGELOG's entry for $VERSION must read '## [$VERSION] - YYYY-MM-DD', got:" >&2
        echo "  $CHANGELOG_LINE" >&2
        exit 1
    fi
    echo "changelog: $CHANGELOG_LINE"
fi

# --- real releases only run from main in CI ---------------------------------
if [ "$dry_run" = "false" ] && [ "${GITHUB_ACTIONS:-}" = "true" ] && [ "${GITHUB_REF:-}" != "refs/heads/main" ]; then
    echo "refusing to release from ${GITHUB_REF:-<no ref>}: releases run from refs/heads/main" >&2
    exit 1
fi

# --- dry run -----------------------------------------------------------------
if [ "$dry_run" = "true" ]; then
    # --allow-dirty: a rehearsal may run on an uncommitted tree; a real
    # publish still requires a clean one.
    echo "cargo publish -p $NAME --locked --dry-run --allow-dirty"
    cargo publish -p "$NAME" --locked --dry-run --allow-dirty
    echo "dry run complete; nothing was published and nothing was tagged"
    exit 0
fi

# --- publish, idempotent ----------------------------------------------------
CRATES_UA="ops-lib-release (+https://github.com/aricma/open-productivity-spec)"
API="https://crates.io/api/v1/crates/$NAME"

CODE=$(curl -s -o /dev/null -w '%{http_code}' -A "$CRATES_UA" "$API")
case "$CODE" in
    200)
        OWNER_REPO=$(curl -sf -A "$CRATES_UA" "$API" | grep -o '"repository":"[^"]*"' | head -n1 | sed -E 's/.*:"([^"]*)"/\1/')
        if [ "$OWNER_REPO" != "$OUR_REPO" ]; then
            echo "crates.io/$NAME is owned by '$OWNER_REPO', not '$OUR_REPO' — refusing to touch it" >&2
            exit 1
        fi
        if curl -sf -A "$CRATES_UA" "$API/$VERSION" >/dev/null; then
            echo "$NAME $VERSION is already on crates.io; skipping publish"
        else
            cargo publish -p "$NAME" --locked
        fi
        ;;
    404)
        cargo publish -p "$NAME" --locked
        ;;
    *)
        echo "crates.io returned HTTP $CODE for $NAME" >&2
        exit 1
        ;;
esac

# --- tag, idempotent --------------------------------------------------------
if [ -n "$(git ls-remote --tags origin "refs/tags/$TAG" 2>/dev/null)" ]; then
    echo "tag $TAG already exists; accepting it"
else
    if [ "${GITHUB_ACTIONS:-}" = "true" ]; then
        git config user.name "github-actions[bot]"
        git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
    fi
    git tag -a "$TAG" -m "ops-lib v$VERSION"
    git push origin "$TAG"
    echo "tagged $TAG"
fi

echo "release complete: $NAME $VERSION"
