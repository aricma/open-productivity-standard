# Releasing ops-lib

The `ops-lib` crate (in `ops/lib/`) is released by a manually triggered GitHub
Actions workflow. The same pipeline can be rehearsed locally with
`mask release`.

## Prerequisites

On `main`, before a release:

1. `version` in `ops/lib/Cargo.toml` is the version to release, in SemVer.
2. `ops/lib/CHANGELOG.md` carries that version as its **latest** entry in
   [Keep a Changelog](https://keepachangelog.com/) form:

   ```markdown
   ## [x.y.z] - YYYY-MM-DD
   ```

   (the top entry today reads `## [0.1.0] - unreleased`; set the date when
   releasing).
3. The repository secret `CARGO_REGISTRY_TOKEN` holds a crates.io API token
   allowed to publish `ops-lib`.

Real releases only run from `main`.

## Trigger the pipeline

GitHub → **Actions** → **Release** → **Run workflow**. Its only input is
`dry_run`:

- **`dry_run = true` (default)** — runs CI and the release preflight, then
  `cargo publish --dry-run`. Nothing is published, nothing is tagged, and
  the changelog requirement is **skipped**, so you can rehearse before the
  changelog entry exists.
- **`dry_run = false`** — the full release: publish, then tag.

The workflow runs on the dispatched commit; for a real release, dispatch
from `main`.

## What the pipeline does

1. **CI** — the shared `.github/actions/ci` composite (fmt, check, clippy,
   tests, docs) followed by `.github/actions/audit` (advisories and license
   policy). Both run inline in the release job.
2. **Release** — `scripts/release.sh`:
   1. read `version` from `ops/lib/Cargo.toml` and log it
   2. validate it as SemVer
   3. *(real runs only)* require `ops/lib/CHANGELOG.md`'s latest released
      version to be this version, formatted `## [x.y.z] - YYYY-MM-DD`
   4. *(real runs only)* skip publishing if this version is already on
      crates.io, otherwise `cargo publish -p ops-lib --locked`
   5. *(real runs only)* skip tagging if `ops-lib/v<version>` already
      exists, otherwise create and push the annotated tag

The pipeline is **idempotent**: re-running a released version succeeds and
does nothing. A publish without a tag, or a tag without a publish, is
healed by the next real run. The workflow serialises via a
`release-ops-lib` concurrency group, so two releases never overlap.

## Tag

```
ops-lib/v<version>
```

For example `ops-lib/v0.1.0` — annotated, pointing at the released commit on
`main`.

## Local rehearsal

`mask release` runs `mask ci` and then the same `scripts/release.sh`:

```sh
mask release              # dry run (default)
mask release --publish    # real publish + tag; needs CARGO_REGISTRY_TOKEN
```

`mask release` is Unix-only; the GitHub pipeline runs on ubuntu.

## Recovery

- **Publish succeeded, tag push failed** — re-run the real release. The
  publish is skipped because the version already exists, and the tag is
  created.
- **Tag exists, publish missing** — re-run the real release. The tag is
  accepted and the missing publish happens.
- **Wrong version released** — yank it on crates.io and cut a new patch
  version. Tags are never moved once pushed.
