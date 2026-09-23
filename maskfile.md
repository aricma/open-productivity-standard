# OPS Actions

This mask file documents the actions you can run in this repo. Install
`mask` (`brew install mask` or `cargo install mask`), then run `mask setup`
once. The Cargo workspace is in `ops/lib/`; the standard is in `specs/`.

Deviations from the standard command set, kept lean on purpose:

- No `preview` — there is no app or server to run, only a library and a
  specification.
- Commands inline their short cargo invocations instead of delegating to
  `scripts/`; only `release` keeps a script, because its preflight and
  idempotency logic is not trivial.
- `setup` verifies the Rust toolchain instead of installing it into
  `tmp/bin`; cargo/rustup tools are host toolchain tools.
- `format` covers the Rust code (`cargo fmt`); the prose has no formatter.
- `test` has no `--watch` flag.
- `mask release` is Unix-only; the GitHub release pipeline runs on ubuntu.

## Targets

- OS/arch: linux/amd64, linux/arm64, darwin/amd64, darwin/arm64, windows/amd64
- Unix shell: sh
- Windows: powershell

## setup

> Verify the Rust toolchain and the cargo subcommands the other actions need.

```sh
set -eu
export PATH="$MASKFILE_DIR/tmp/bin:$PATH"
missing=0
for tool in cargo rustc rustfmt cargo-clippy cargo-audit cargo-deny; do
    if command -v "$tool" >/dev/null 2>&1; then
        echo "  ok       $tool"
    else
        echo "  MISSING  $tool"
        missing=1
    fi
done
if [ "$missing" -ne 0 ]; then
    echo
    echo "Install Rust:        https://rustup.rs"
    echo "Add components:      rustup component add rustfmt clippy"
    echo "Add cargo commands:  cargo install cargo-audit cargo-deny"
    exit 1
fi
echo "All tools present."
```

```powershell
$env:PATH = "$env:MASKFILE_DIR\tmp\bin;$env:PATH"
$missing = 0
foreach ($tool in "cargo", "rustc", "rustfmt", "cargo-clippy", "cargo-audit", "cargo-deny") {
    if (Get-Command $tool -ErrorAction SilentlyContinue) {
        Write-Host "  ok       $tool"
    }
    else {
        Write-Host "  MISSING  $tool"
        $missing = 1
    }
}
if ($missing -ne 0) {
    Write-Host ""
    Write-Host "Install Rust:        https://rustup.rs"
    Write-Host "Add components:      rustup component add rustfmt clippy"
    Write-Host "Add cargo commands:  cargo install cargo-audit cargo-deny"
    exit 1
}
Write-Host "All tools present."
```

## format

> Check formatting of the Rust code. Pass --fix to rewrite it.

**OPTIONS**

- fix
  - flags: -f --fix
  - desc: Rewrite files instead of only checking them

```sh
set -eu
export PATH="$MASKFILE_DIR/tmp/bin:$PATH"
if [ "${fix:-}" = "true" ]; then
    cargo fmt
else
    cargo fmt --check
fi
```

```powershell
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true
$env:PATH = "$env:MASKFILE_DIR\tmp\bin;$env:PATH"
if ($fix) { cargo fmt } else { cargo fmt --check }
```

## build

> Lint, compile, build docs, and scan dependencies. Pass --fix to auto-fix lints.

**OPTIONS**

- fix
  - flags: -f --fix
  - desc: Auto-fix clippy lints where possible

```sh
set -eu
export PATH="$MASKFILE_DIR/tmp/bin:$PATH"
if [ "${fix:-}" = "true" ]; then
    cargo clippy --fix --allow-dirty --all-targets --all-features -- -D warnings
fi
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo audit
cargo deny check
```

```powershell
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true
$env:PATH = "$env:MASKFILE_DIR\tmp\bin;$env:PATH"
if ($fix) { cargo clippy --fix --allow-dirty --all-targets --all-features -- -D warnings }
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
$env:RUSTDOCFLAGS = "-D warnings"
cargo doc --no-deps --all-features
cargo audit
cargo deny check
```

## test

> Run the test suite for a profile.

**OPTIONS**

- profile
  - flags: -p --profile
  - type: string
  - choices: essential, security, performance, acceptance, all
  - desc: Test profile to run (acceptance = OPS conformance suite only)

```sh
set -eu
export PATH="$MASKFILE_DIR/tmp/bin:$PATH"
case "${profile:-essential}" in
essential) cargo test --all-targets --all-features ;;
acceptance) cargo test --test ops_conformance ;;
security) cargo audit && cargo deny check ;;
performance) echo "no performance tests yet (see TODOs.md)" ;;
all)
    cargo test --all-targets --all-features
    cargo test --test ops_conformance
    cargo audit && cargo deny check
    echo "no performance tests yet (see TODOs.md)"
    ;;
esac
```

```powershell
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true
$env:PATH = "$env:MASKFILE_DIR\tmp\bin;$env:PATH"
if (-not $profile) { $profile = "essential" }
switch ($profile) {
    "essential" { cargo test --all-targets --all-features }
    "acceptance" { cargo test --test ops_conformance }
    "security" { cargo audit; cargo deny check }
    "performance" { Write-Host "no performance tests yet (see TODOs.md)" }
    "all" {
        cargo test --all-targets --all-features
        cargo test --test ops_conformance
        cargo audit
        cargo deny check
        Write-Host "no performance tests yet (see TODOs.md)"
    }
}
```

## ci

> Run format, build, and test as a local CI check, fail-fast.

**OPTIONS**

- test_profile
  - flags: --test-profile
  - type: string
  - choices: essential, security, performance, acceptance, all
  - desc: Profile passed to the test step

```sh
$MASK format && $MASK build && $MASK test --profile "${test_profile:-all}"
```

```powershell
$selected = if ($test_profile) { $test_profile } else { "all" }
& $env:MASK format; if ($LASTEXITCODE) { exit $LASTEXITCODE }
& $env:MASK build; if ($LASTEXITCODE) { exit $LASTEXITCODE }
& $env:MASK test --profile $selected; if ($LASTEXITCODE) { exit $LASTEXITCODE }
```

## release

> Rehearse or run the release locally: CI, preflight, then publish.

**OPTIONS**

- mode
  - flags: -m --mode
  - type: string
  - choices: dry-run, publish
  - desc: dry-run (default) rehearses; publish pushes to crates.io and tags

```sh
set -eu
export PATH="$MASKFILE_DIR/tmp/bin:$PATH"
$MASK ci
sh "$MASKFILE_DIR/scripts/release.sh" "--${mode:-dry-run}"
```
