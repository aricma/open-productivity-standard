# Corpus Test Codegen Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn every `test-corpus/` case into its own named, self-describing `#[test]` in `cargo test --test corpus`, replacing the three loop-based tests that bury individual cases.

**Architecture:** A `build.rs` in `ops/` walks `../test-corpus` at build time and emits one `#[test] fn` per case into `OUT_DIR/corpus_cases.rs`; `ops/tests/corpus.rs` shrinks to a doc comment plus an `include!` of that file. Test names are `<group>__<path>__<claim_slug>` (claim slug lowercased, non-alphanumeric runs collapsed to `_`, truncated to 120 chars). Unsupported-format cases become `#[ignore = "unsupported format: <fmt>"]` tests so skips stay visible and counted.

**Tech Stack:** Rust 2024, std only (no new dependencies), Cargo build scripts.

**Depends on:** none.

**Spec:** `tmp/superpowers/specs/2026-09-24-corpus-test-codegen-design.md`

---

## File Structure

- Create: `ops/build.rs` — walks the corpus, emits generated tests
- Modify: `ops/tests/corpus.rs` — reduced to a shim (`include!` + one helper)
- Unchanged: `ops/Cargo.toml` (build scripts are auto-detected), `ops/lib/`, `ops/tests/model.rs`, `test-corpus/`

---

### Task 1: `ops/build.rs`

**Files:**
- Create: `ops/build.rs`

- [ ] **Step 1: Write the build script**

```rust
//! Generates one `#[test]` per test-corpus case into `OUT_DIR/corpus_cases.rs`,
//! which `tests/corpus.rs` includes.
//!
//! A plain macro cannot enumerate the filesystem, so case discovery lives
//! here. Names are `<group>__<path>__<claim_slug>` so a case can be found by
//! path and understood from its name; unsupported formats become
//! `#[ignore]`d tests (skips stay visible and counted).

use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// The crate's `Format::from_name` (see `ops/lib/v0/format.rs`). Duplicated
/// here because a build script cannot depend on the crate it is building.
/// `ponytail:` keep in sync when a format is added.
fn format_variant(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "json" => "Json",
        "yaml" | "yml" => "Yaml",
        "jsonl" | "ndjson" => "Jsonl",
        _ => return None,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Group {
    Valid,
    Invalid,
    Convert,
    Roundtrips,
}

struct Case {
    /// Path under the corpus root, `/`-separated, including the group
    /// component (e.g. `valid/foo`).
    rel: String,
    group: Group,
    claim: String,
    given_ext: String,
    expectation_ext: Option<String>,
}

fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
}

fn case_at(dir: &Path, corpus: &Path) -> Option<Case> {
    let mut claim = None;
    let mut given = None;
    let mut expectation = None;
    for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())) {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if name == "claim.txt" {
            claim = Some(path);
        } else if name.starts_with("given.") {
            assert!(given.is_none(), "{}: more than one given file", dir.display());
            given = Some(path);
        } else if name.starts_with("expectation.") {
            assert!(
                expectation.is_none(),
                "{}: more than one expectation file",
                dir.display()
            );
            expectation = Some(path);
        }
    }

    let (claim, given) = (claim?, given?);
    let rel = dir
        .strip_prefix(corpus)
        .unwrap_or(dir)
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    let group = match rel.split('/').next().unwrap_or("") {
        "valid" => Group::Valid,
        "invalid" => Group::Invalid,
        "convert" => Group::Convert,
        "roundtrips" => Group::Roundtrips,
        other => panic!("{rel}: not under valid/, invalid/, convert/, or roundtrips/ ({other})"),
    };
    if matches!(group, Group::Convert | Group::Roundtrips) {
        assert!(expectation.is_some(), "{rel}: needs an expectation.<ext>");
    }
    Some(Case {
        rel,
        group,
        claim: fs::read_to_string(&claim)
            .unwrap_or_else(|e| panic!("read {}: {e}", claim.display()))
            .trim()
            .to_string(),
        given_ext: extension(&given).unwrap_or_else(|| panic!("{dir:?}: given has no extension")),
        expectation_ext: expectation.as_ref().and_then(|p| extension(p)),
    })
}

fn collect(dir: &Path, corpus: &Path, out: &mut Vec<Case>) {
    if let Some(case) = case_at(dir, corpus) {
        out.push(case);
    }
    let mut children: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .filter_map(|entry| {
            let path = entry.expect("dir entry").path();
            path.is_dir().then_some(path)
        })
        .collect();
    children.sort();
    for child in children {
        collect(&child, corpus, out);
    }
}

fn slug(s: &str) -> String {
    let mut out = String::new();
    let mut last_was_sep = false;
    for c in s.to_lowercase().chars() {
        if c.is_alphanumeric() {
            out.push(c);
            last_was_sep = false;
        } else if !last_was_sep {
            out.push('_');
            last_was_sep = true;
        }
    }
    out.trim_end_matches('_').to_string()
}

fn test_name(case: &Case) -> String {
    let group = match case.group {
        Group::Valid => "valid",
        Group::Invalid => "invalid",
        Group::Convert => "convert",
        Group::Roundtrips => "roundtrips",
    };
    // Path *under* the group, so the group isn't repeated in the name.
    let path_slug = slug(case.rel.splitn(2, '/').nth(1).unwrap_or(""));
    let mut claim = slug(&case.claim);
    claim.truncate(120);
    format!("{group}__{path_slug}__{claim}")
}

fn gen_body(case: &Case) -> String {
    let given_variant = format_variant(&case.given_ext)
        .unwrap_or_else(|| panic!("{}: unsupported given format .{}", case.rel, case.given_ext));
    let read_given = format!(
        "read_text(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/../test-corpus/{}/given.{}\"))",
        case.rel, case.given_ext
    );
    match case.group {
        Group::Valid => format!(
            r#"    let given = {read_given};
    ops_lib::v0::read(ops_lib::v0::Format::{given_variant}, &given)
        .unwrap_or_else(|e| panic!("{rel}: must be accepted — {claim}: {{e}}"));
"#,
            rel = case.rel,
            claim = case.claim
        ),
        Group::Invalid => format!(
            r#"    let given = {read_given};
    assert!(
        ops_lib::v0::read(ops_lib::v0::Format::{given_variant}, &given).is_err(),
        "{rel}: must be rejected — {claim}"
    );
"#,
            rel = case.rel,
            claim = case.claim
        ),
        Group::Convert | Group::Roundtrips => {
            let exp = case.expectation_ext.as_ref().expect("has an expectation");
            let exp_variant = format_variant(exp)
                .unwrap_or_else(|| panic!("{}: unsupported expectation format .{}", case.rel, exp));
            let read_expected = format!(
                "read_text(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/../test-corpus/{}/expectation.{}\"))",
                case.rel, exp
            );
            format!(
                r#"    let given = {read_given};
    let expected = {read_expected};
    let tasks = ops_lib::v0::read(ops_lib::v0::Format::{given_variant}, &given)
        .unwrap_or_else(|e| panic!("{rel}: given rejected: {{e}}"));
    let out = ops_lib::v0::write(ops_lib::v0::Format::{exp_variant}, &tasks)
        .unwrap_or_else(|e| panic!("{rel}: write failed: {{e}}"));
    let produced = ops_lib::v0::read(ops_lib::v0::Format::{exp_variant}, &out)
        .unwrap_or_else(|e| panic!("{rel}: output rejected: {{e}}"));
    let wanted = ops_lib::v0::read(ops_lib::v0::Format::{exp_variant}, &expected)
        .unwrap_or_else(|e| panic!("{rel}: expectation rejected: {{e}}"));
    assert_eq!(
        produced,
        wanted,
        "{rel}: {claim} (the exported model differs from the expectation)"
    );
"#,
                rel = case.rel,
                claim = case.claim
            )
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=../test-corpus");
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("../test-corpus");
    let mut cases = Vec::new();
    collect(&corpus, &corpus, &mut cases);
    assert!(!cases.is_empty(), "no cases found in {}", corpus.display());

    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut generated = String::new();
    for case in &cases {
        let base = test_name(case);
        let n = seen.entry(base.clone()).or_insert(0);
        *n += 1;
        let name = if *n == 1 { base } else { format!("{base}_{n}") };

        let given_ok = format_variant(&case.given_ext).is_some();
        let exp_ok = case
            .expectation_ext
            .as_deref()
            .is_none_or(|e| format_variant(e).is_some());
        if given_ok && exp_ok {
            writeln!(
                generated,
                "#[test]\nfn {name}() {{\n{}\n}}\n",
                gen_body(case)
            )
            .unwrap();
        } else {
            let reason = if given_ok {
                case.expectation_ext.as_deref().unwrap_or("")
            } else {
                case.given_ext.as_str()
            };
            writeln!(
                generated,
                "#[test]\n#[ignore = \"unsupported format: {reason}\"]\nfn {name}() {{}}\n"
            )
            .unwrap();
        }
    }

    let out_dir = std::env::var_os("OUT_DIR").expect("OUT_DIR");
    let dest = Path::new(&out_dir).join("corpus_cases.rs");
    fs::write(&dest, generated).unwrap_or_else(|e| panic!("write {}: {e}", dest.display()));
}
```

- [ ] **Step 2: Build once**

Run: `cargo build -p ops-lib`
Expected: succeeds; `ops/target/debug/build/ops-lib-*/out/corpus_cases.rs` exists with ~150 `fn` definitions.

- [ ] **Step 3: Commit**

```bash
git add ops/build.rs
git commit -m "test(corpus): generate one test per case via build script"
```

---

### Task 2: `ops/tests/corpus.rs` shim

**Files:**
- Modify: `ops/tests/corpus.rs`

- [ ] **Step 1: Replace the file**

```rust
//! Runs the language-agnostic corpus in `../test-corpus`, one `#[test]` per
//! case. Cases are discovered and their test functions generated at build
//! time by `build.rs`; this file only includes the result.
//!
//! - `valid/<case>` — importing `given` must succeed.
//! - `invalid/<case>` — importing `given` must fail; the spec does not
//!   prescribe a reason, so none is matched.
//! - `convert/<case>` and `roundtrips/<case>` — read `given`, write with the
//!   expectation's format, read back, and require the model to equal the
//!   model of `expectation.<ext>`. Comparison is semantic.
//!
//! Formats the crate does not implement are generated as `#[ignore]`d tests.

fn read_text(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"))
}

include!(concat!(env!("OUT_DIR"), "/corpus_cases.rs"));
```

- [ ] **Step 2: Run the corpus**

Run: `cargo test -p ops-lib --test corpus`
Expected: PASS, listing ~150 individually-named tests plus ignored ones (CSV/Markdown).

Run: `cargo test -p ops-lib --test corpus convert__json_to_yaml`
Expected: PASS, exactly one test matches.

- [ ] **Step 3: Commit**

```bash
git add ops/tests/corpus.rs
git commit -m "test(corpus): include generated per-case tests"
```

---

### Task 3: Final verification

- [ ] **Step 1: Full gate**

Run: `mask ci`
Expected: format, clippy `-D warnings` (generated code included, so it must be clean), docs, tests, audit, deny all pass.

- [ ] **Step 2: Skip reporting**

Run: `cargo test -p ops-lib --test corpus -- --ignored --list`
Expected: the CSV/Markdown cases listed with "unsupported format: csv"/"md".

- [ ] **Step 3: No stale runtime collector**

Run: `grep -n "collect\|corpus()" ops/tests/corpus.rs`
Expected: no output.

---

## Self-review notes

- The build script duplicates `Format::from_name`'s extension map; it cannot
  import the lib crate, and the duplication is three formats plus a
  `ponytail:` comment pointing back at the source of truth.
- `cargo fmt --check` does not inspect OUT_DIR, but clippy `-D warnings`
  does lint the included generated code, so generated bodies must be clean.
- The "convert/roundtrips needs an expectation" assert moved from test time
  to build time (fails `cargo build` with the case name).
- `env!("CARGO_MANIFEST_DIR")` in the generated code expands in the test
  crate to `ops/`, so corpus paths resolve regardless of working directory
  (same trick as the old `CORPUS` const).
- Test names are valid Rust identifiers: they start with a group name (a
  letter), and every later character is alphanumeric or `_`.
