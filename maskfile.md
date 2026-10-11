# OPS Actions

This mask file documents the actions you can run in this repo. Install
`mask` (`brew install mask` or `cargo install mask`), then run `mask setup`
once. The Cargo workspace is in `ops/`; the standard is in `specs/`.

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
# A reader for a small model has no business being megabytes. The crate is
# ~1.3 MiB today, so 2 MiB leaves room for its own code plus what `regex`
# and `serde` monomorphize into it, and trips on an accidental heavyweight
# dependency.
cargo build --release -p ops-lib
size=$(wc -c < target/release/libops_lib.rlib | tr -d ' ')
budget=$((2 * 1024 * 1024))
printf 'ops-lib rlib: %s bytes (budget %s)\n' "$size" "$budget"
if [ "$size" -gt "$budget" ]; then
    echo 'the compiled library outgrew its budget' >&2
    exit 1
fi
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
# See the sh block for why the budget is 2 MiB.
cargo build --release -p ops-lib
$size = (Get-Item target/release/ops_lib.rlib).Length
$budget = 2 * 1024 * 1024
Write-Host "ops-lib rlib: $size bytes (budget $budget)"
if ($size -gt $budget) { Write-Error 'the compiled library outgrew its budget'; exit 1 }
```

## gen-corpus

> Regenerate `ops/tests/acceptance/` from `specs/<version>/tests/`. Pass --check to verify only.

**OPTIONS**

- check
  - flags: --check
  - desc: Fail if the committed generated tests are out of date; write nothing

```sh
set -eu
export PATH="$MASKFILE_DIR/tmp/bin:$PATH"
if [ "${check:-}" = "true" ]; then
    cargo run -q -p gen-corpus -- --check
else
    cargo run -q -p gen-corpus
fi
```

```powershell
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true
$env:PATH = "$env:MASKFILE_DIR\tmp\bin;$env:PATH"
if ($check) {
    cargo run -q -p gen-corpus -- --check
}
else {
    cargo run -q -p gen-corpus
}
```

## test

> Run the test suite for a profile.

**OPTIONS**

- profile
  - flags: -p --profile
  - type: string
  - choices: essential, security, performance, acceptance, all
  - desc: Test profile to run (acceptance = the portable corpus only)

```sh
set -eu
export PATH="$MASKFILE_DIR/tmp/bin:$PATH"
$MASK gen-corpus
case "${profile:-essential}" in
essential)
    cargo test --lib --test essentials
    cargo test -p gen-corpus
    cargo test --doc
    ;;
acceptance) cargo test -p ops-lib --test acceptance ;;
security) cargo test -p ops-lib --test security ;;
performance) cargo test -p ops-lib --test performance -- --nocapture ;;
all)
    cargo test --lib --test essentials
    cargo test -p gen-corpus
    cargo test --doc
    cargo test -p ops-lib --test acceptance
    cargo test -p ops-lib --test security
    cargo test -p ops-lib --test performance -- --nocapture
    ;;
esac
```

```powershell
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true
$env:PATH = "$env:MASKFILE_DIR\tmp\bin;$env:PATH"
& $env:MASK gen-corpus; if ($LASTEXITCODE) { exit $LASTEXITCODE }
if (-not $profile) { $profile = "essential" }
switch ($profile) {
    "essential" {
        cargo test --lib --test essentials
        cargo test -p gen-corpus
        cargo test --doc
    }
    "acceptance" { cargo test -p ops-lib --test acceptance }
    "security" { cargo test -p ops-lib --test security }
    "performance" { cargo test -p ops-lib --test performance -- --nocapture }
    "all" {
        cargo test --lib --test essentials
        cargo test -p gen-corpus
        cargo test --doc
        cargo test -p ops-lib --test acceptance
        cargo test -p ops-lib --test security
        cargo test -p ops-lib --test performance -- --nocapture
    }
}
```

## ci

> Run format, build, and test as a local CI check, fail-fast. Pass --fix to auto-fix first.

**OPTIONS**

- test_profile
  - flags: --test-profile
  - type: string
  - choices: essential, security, performance, acceptance, all
  - desc: Profile passed to the test step
- fix
  - flags: -f --fix
  - desc: Auto-fix formatting and lints before checking

```sh
fix_arg=""
if [ "${fix:-}" = "true" ]; then
    fix_arg="--fix"
fi
$MASK format $fix_arg && $MASK build $fix_arg && $MASK test --profile "${test_profile:-all}"
```

```powershell
$selected = if ($test_profile) { $test_profile } else { "all" }
$fixArg = if ($fix) { @("--fix") } else { @() }
& $env:MASK format @fixArg; if ($LASTEXITCODE) { exit $LASTEXITCODE }
& $env:MASK build @fixArg; if ($LASTEXITCODE) { exit $LASTEXITCODE }
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
