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
- `test` has a `features` profile the standard set does not: a cargo feature
  is fixed per compilation, so a subset of them can only be tested by building
  one.
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
# Every member, so the tools are formatted with the library.
if [ "${fix:-}" = "true" ]; then
    cargo fmt --all
else
    cargo fmt --all --check
fi
```

```powershell
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true
$env:PATH = "$env:MASKFILE_DIR\tmp\bin;$env:PATH"
if ($fix) { cargo fmt --all } else { cargo fmt --all --check }
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
# Every member: the library and the tools that generate its tests.
if [ "${fix:-}" = "true" ]; then
    cargo clippy --workspace --fix --allow-dirty --all-targets --all-features -- -D warnings
fi
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
# Every format is a cargo feature, and each has to build on its own: a format
# that only compiles beside another is not behind a feature, and the code of a
# format that is off should be gone rather than merely unused.
for feature in json yaml jsonl csv; do
    cargo clippy -q -p ops-lib --no-default-features --features "$feature" --all-targets -- -D warnings
    echo "ops-lib builds with only $feature"
done
# The crate is a reader and a writer, so with no format it refuses to build,
# and says which ones there are.
if cargo check -q -p ops-lib --no-default-features 2>/dev/null; then
    echo 'ops-lib built with no format enabled' >&2
    exit 1
fi
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
cargo audit
cargo deny check
# A reader for a small model has no business being megabytes. The crate is
# ~1.3 MiB today, so 2 MiB leaves room for its own code plus what `regex`
# and `serde` monomorphize into it, and trips on an accidental heavyweight
# dependency. This block runs where that holds: 1 282 432 bytes on macOS and
# 1 679 352 on Linux, against 2 320 838 on Windows, where the object format
# records more around the same code. The PowerShell block has the rest.
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
# See the sh block: every member, the library and the tools.
if ($fix) { cargo clippy --workspace --fix --allow-dirty --all-targets --all-features -- -D warnings }
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
# See the sh block: every format builds alone, and no format does not build.
foreach ($feature in "json", "yaml", "jsonl", "csv") {
    cargo clippy -q -p ops-lib --no-default-features --features $feature --all-targets -- -D warnings
    Write-Host "ops-lib builds with only $feature"
}
$noFormatBuilt = $true
try { cargo check -q -p ops-lib --no-default-features }
catch { $noFormatBuilt = $false }
if ($noFormatBuilt) {
    Write-Error 'ops-lib built with no format enabled'
    exit 1
}
$env:RUSTDOCFLAGS = "-D warnings"
cargo doc --workspace --no-deps --all-features
cargo audit
cargo deny check
# See the sh block for the budget. Windows needs 3 MiB because of the object
# format, not our code: the code, data and unwind tables come to ~465 KB on
# every platform, but ELF and COFF keep a section, a symbol and relocations
# per function so the linker can drop dead code, where Mach-O merges code and
# lets the linker split on symbols. Measured: 2 320 838 bytes here against
# 1 282 432 on macOS and 1 679 352 on Linux.
cargo build --release -p ops-lib
$size = (Get-Item target/release/ops_lib.rlib).Length
$budget = 3 * 1024 * 1024
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
  - choices: essential, security, performance, acceptance, features, all
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
features)
    # One cargo invocation per format. A feature set is fixed per compilation,
    # so a subset of them can only be tested by building one.
    for feature in json yaml jsonl csv; do
        cargo test -q -p ops-lib --no-default-features --features "$feature"
        echo "ops-lib passes with only $feature"
    done
    ;;
acceptance) cargo test -p ops-lib --test acceptance ;;
security) cargo test -p ops-lib --test security ;;
performance)
    # The envelope, printed so a log says what machine produced the numbers.
    # One test thread: the harness would otherwise run these tests in parallel
    # and time them against each other, which makes the result depend on how
    # many cores the machine has.
    cores=$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo "?")
    limit=""
    # A container's CPU limit is a time quota, not a core count, so the visible
    # core count above stays the host's: print the quota that actually applies.
    if [ -r /sys/fs/cgroup/cpu.max ]; then
        limit=", cpu.max $(cat /sys/fs/cgroup/cpu.max)"
    fi
    if [ -r /sys/fs/cgroup/memory.max ]; then
        limit="$limit, memory limit $(cat /sys/fs/cgroup/memory.max)"
    fi
    echo "performance profile: $(uname -srm), $cores cores$limit, one test thread"
    cargo test -p ops-lib --test performance -- --nocapture --test-threads=1
    ;;
all)
    cargo test --lib --test essentials
    cargo test -p gen-corpus
    cargo test --doc
    cargo test -p ops-lib --test acceptance
    cargo test -p ops-lib --test security
    $MASK test --profile performance
    $MASK test --profile features
    ;;
esac
```

```powershell
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $true
$env:PATH = "$env:MASKFILE_DIR\tmp\bin;$env:PATH"
# See the sh block: regenerate the corpus first. $env:MASK is the string
# "mask --maskfile <path>", which PowerShell cannot invoke as a command, so
# call the parts instead.
$maskfile = $env:MASK -replace '^mask --maskfile ', ''
& mask --maskfile $maskfile gen-corpus; if ($LASTEXITCODE) { exit $LASTEXITCODE }
if (-not $profile) { $profile = "essential" }
switch ($profile) {
    "essential" {
        cargo test --lib --test essentials
        cargo test -p gen-corpus
        cargo test --doc
    }
    "features" {
        # See the sh block: one cargo invocation per format.
        foreach ($feature in "json", "yaml", "jsonl", "csv") {
            cargo test -q -p ops-lib --no-default-features --features $feature
            Write-Host "ops-lib passes with only $feature"
        }
    }
    "acceptance" { cargo test -p ops-lib --test acceptance }
    "security" { cargo test -p ops-lib --test security }
    "performance" {
        # See the sh block: the envelope, and one test thread.
        $cores = [Environment]::ProcessorCount
        Write-Host "performance profile: $([Environment]::OSVersion.VersionString), $cores cores, one test thread"
        cargo test -p ops-lib --test performance -- --nocapture --test-threads=1
    }
    "all" {
        cargo test --lib --test essentials
        cargo test -p gen-corpus
        cargo test --doc
        cargo test -p ops-lib --test acceptance
        cargo test -p ops-lib --test security
        & mask --maskfile $maskfile test --profile performance; if ($LASTEXITCODE) { exit $LASTEXITCODE }
        & mask --maskfile $maskfile test --profile features; if ($LASTEXITCODE) { exit $LASTEXITCODE }
    }
}
```

## ci

> Run format, build, and test as a local CI check, fail-fast. Pass --fix to auto-fix first.

**OPTIONS**

- test_profile
  - flags: --test-profile
  - type: string
  - choices: essential, security, performance, acceptance, features, all
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
# $env:MASK is the string "mask --maskfile <path>", which PowerShell cannot
# invoke as a command, so call the parts instead.
$maskfile = $env:MASK -replace '^mask --maskfile ', ''
$selected = if ($test_profile) { $test_profile } else { "all" }
$fixArg = if ($fix) { @("--fix") } else { @() }
& mask --maskfile $maskfile format @fixArg; if ($LASTEXITCODE) { exit $LASTEXITCODE }
& mask --maskfile $maskfile build @fixArg; if ($LASTEXITCODE) { exit $LASTEXITCODE }
& mask --maskfile $maskfile test --profile $selected; if ($LASTEXITCODE) { exit $LASTEXITCODE }
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
