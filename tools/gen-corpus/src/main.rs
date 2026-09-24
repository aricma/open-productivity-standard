//! Generates `ops/tests/acceptance/**` — one `#[test]` per test-corpus case —
//! so the suite is committed, reviewable, and mirrors the corpus layout.
//!
//! `valid/` and `invalid/` are split one module per input format; `convert/`
//! and `roundtrips/` are single files (their cases are cross-format). The test
//! name is the case directory name. Formats the crate does not implement become
//! `#[ignore]`d tests, so skips stay visible and counted.
//!
//! ```text
//! cargo run -p gen-corpus             regenerate, removing stale files
//! cargo run -p gen-corpus -- --check  fail if the committed files differ
//! ```
//!
//! The generator owns `ops/tests/acceptance/`: it writes every file it lists
//! and removes any other `.rs` in that directory.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// The crate's `Format::from_name` (see `ops/lib/v0/format.rs`), duplicated
/// because this is a separate crate. `ponytail:` keep in sync when a format is
/// added.
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

impl Group {
    fn dir(self) -> &'static str {
        match self {
            Group::Valid => "valid",
            Group::Invalid => "invalid",
            Group::Convert => "convert",
            Group::Roundtrips => "roundtrips",
        }
    }
}

struct Case {
    /// Path under the corpus root, `/`-separated, including the group
    /// component (e.g. `valid/json/foo`).
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
            assert!(
                given.is_none(),
                "{}: more than one given file",
                dir.display()
            );
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

/// The test name is the case directory name, snake_cased: the group and
/// format are already the module path (`valid::json::foo`).
fn test_name(case: &Case) -> String {
    slug(case.rel.rsplit('/').next().unwrap_or(&case.rel))
}

fn gen_body(case: &Case) -> String {
    let given_variant = format_variant(&case.given_ext)
        .unwrap_or_else(|| panic!("{}: unsupported given format .{}", case.rel, case.given_ext));
    // `valid/` and `invalid/` are nested one level deeper than `convert`/`roundtrips`.
    let prefix = match case.group {
        Group::Valid | Group::Invalid => "../../../../test-corpus",
        Group::Convert | Group::Roundtrips => "../../../test-corpus",
    };
    let read_given = format!(
        "include_str!(\"{prefix}/{}/given.{}\")",
        case.rel, case.given_ext
    );
    match case.group {
        Group::Valid => format!(
            r#"    let given = {read_given};
    ops_lib::v0::read(ops_lib::v0::Format::{given_variant}, given)
        .unwrap_or_else(|e| panic!("{rel}: must be accepted — {claim}: {{e}}"));
"#,
            read_given = read_given,
            given_variant = given_variant,
            rel = case.rel,
            claim = case.claim,
        ),
        Group::Invalid => format!(
            r#"    let given = {read_given};
    assert!(
        ops_lib::v0::read(ops_lib::v0::Format::{given_variant}, given).is_err(),
        "{rel}: must be rejected — {claim}"
    );
"#,
            read_given = read_given,
            given_variant = given_variant,
            rel = case.rel,
            claim = case.claim,
        ),
        Group::Convert | Group::Roundtrips => {
            let exp = case.expectation_ext.as_ref().expect("has an expectation");
            let exp_variant = format_variant(exp)
                .unwrap_or_else(|| panic!("{}: unsupported expectation format .{}", case.rel, exp));
            let read_expected = format!(
                "include_str!(\"{prefix}/{}/expectation.{}\")",
                case.rel, exp
            );
            format!(
                r#"    let given = {read_given};
    let expected = {read_expected};
    let tasks = ops_lib::v0::read(ops_lib::v0::Format::{given_variant}, given)
        .unwrap_or_else(|e| panic!("{rel}: given rejected: {{e}}"));
    let out = ops_lib::v0::write(ops_lib::v0::Format::{exp_variant}, &tasks)
        .unwrap_or_else(|e| panic!("{rel}: write failed: {{e}}"));
    let produced = ops_lib::v0::read(ops_lib::v0::Format::{exp_variant}, &out)
        .unwrap_or_else(|e| panic!("{rel}: output rejected: {{e}}"));
    let wanted = ops_lib::v0::read(ops_lib::v0::Format::{exp_variant}, expected)
        .unwrap_or_else(|e| panic!("{rel}: expectation rejected: {{e}}"));
    assert_eq!(
        produced, wanted,
        "{rel}: {claim} (the exported model differs from the expectation)"
    );
"#,
                read_given = read_given,
                read_expected = read_expected,
                given_variant = given_variant,
                exp_variant = exp_variant,
                rel = case.rel,
                claim = case.claim,
            )
        }
    }
}

/// Emits the `#[test]` blocks for an iterator of cases, ending with a single
/// trailing newline.
fn emit_tests<'a>(cases: impl Iterator<Item = &'a Case>) -> String {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut out = String::new();
    for case in cases {
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
                out,
                "#[rustfmt::skip]\n#[test]\nfn {name}() {{\n{}}}\n",
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
                out,
                "#[rustfmt::skip]\n#[test]\n#[ignore = \"unsupported format: {reason}\"]\nfn {name}() {{}}\n"
            )
            .unwrap();
        }
    }
    if !out.is_empty() {
        out.pop();
    }
    out
}

fn header(summary: &str) -> String {
    format!("//! {summary} Generated by `gen-corpus`; do not edit by hand.\n\n")
}

const MAIN: &str = r#"//! Runs the language-agnostic corpus in `../../../test-corpus`, one `#[test]`
//! per case, organized by corpus group. `valid/` and `invalid/` are split one
//! module per format (`json`, `yaml`, `jsonl`, `csv`, `md`); `convert` and
//! `roundtrips` are single files.
//!
//! - `valid` — importing `given` must succeed.
//! - `invalid` — importing `given` must fail; the spec does not prescribe a
//!   reason, so none is matched.
//! - `convert` / `roundtrips` — read `given`, write with the expectation's
//!   format, read back, and require the model to equal the model of
//!   `expectation.<ext>`. Comparison is semantic.
//!
//! Formats the crate does not implement are generated as `#[ignore]`d tests.
//!
//! Generated by `gen-corpus`; do not edit by hand.

mod convert;
mod invalid;
mod roundtrips;
mod valid;
"#;

/// The generated files, as `(path relative to the acceptance dir, content)`.
fn generate(corpus: &Path) -> Vec<(String, String)> {
    let mut cases = Vec::new();
    collect(corpus, corpus, &mut cases);
    assert!(!cases.is_empty(), "no cases found in {}", corpus.display());

    let mut files = vec![("main.rs".to_string(), MAIN.to_string())];

    // Single-file groups: convert and roundtrips (cross-format).
    for group in [Group::Convert, Group::Roundtrips] {
        let dir = group.dir();
        let mut content = header(&format!("One `#[test]` per `test-corpus/{dir}/` case."));
        content.push_str(&emit_tests(cases.iter().filter(|c| c.group == group)));
        files.push((format!("{dir}.rs"), content));
    }

    // Per-format groups: valid and invalid, one module per input format.
    for group in [Group::Valid, Group::Invalid] {
        let dir = group.dir();
        let mut exts: Vec<&str> = cases
            .iter()
            .filter(|c| c.group == group)
            .map(|c| c.given_ext.as_str())
            .collect();
        exts.sort_unstable();
        exts.dedup();

        let mut mod_rs = header(&format!(
            "One `#[test]` per `test-corpus/{dir}/` case, split by format."
        ));
        for ext in &exts {
            let mut content = header(&format!("`test-corpus/{dir}/` cases with `given.{ext}`."));
            content.push_str(&emit_tests(
                cases
                    .iter()
                    .filter(|c| c.group == group && c.given_ext == *ext),
            ));
            files.push((format!("{dir}/{ext}.rs"), content));
            mod_rs.push_str(&format!("mod {ext};\n"));
        }
        files.push((format!("{dir}/mod.rs"), mod_rs));
    }

    files
}

/// Every `.rs` file under `dir`, recursively.
fn rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(rs_files(&path));
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    out
}

fn write(out: &Path, files: &[(String, String)]) -> ExitCode {
    let mut managed = HashSet::new();
    for (rel, content) in files {
        let path = out.join(rel);
        if let Some(parent) = path.parent()
            && let Err(e) = fs::create_dir_all(parent)
        {
            eprintln!("mkdir {}: {e}", parent.display());
            return ExitCode::FAILURE;
        }
        if let Err(e) = fs::write(&path, content) {
            eprintln!("write {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
        managed.insert(path);
    }
    for path in rs_files(out) {
        if !managed.contains(&path) {
            if let Err(e) = fs::remove_file(&path) {
                eprintln!("remove {}: {e}", path.display());
                return ExitCode::FAILURE;
            }
            let rel = path.strip_prefix(out).unwrap_or(&path);
            println!("removed stale {}", rel.display());
        }
    }
    println!("wrote {} generated files", files.len());
    ExitCode::SUCCESS
}

fn check(out: &Path, files: &[(String, String)]) -> ExitCode {
    let mut stale = Vec::new();
    let mut managed = HashSet::new();
    for (rel, content) in files {
        let path = out.join(rel);
        managed.insert(path.clone());
        match fs::read_to_string(&path) {
            Ok(on_disk) if on_disk == *content => {}
            Ok(_) => stale.push(format!("out of date: {rel}")),
            Err(_) => stale.push(format!("missing:     {rel}")),
        }
    }
    for path in rs_files(out) {
        if !managed.contains(&path) {
            let rel = path.strip_prefix(out).unwrap_or(&path);
            stale.push(format!("stale:       {}", rel.display()));
        }
    }
    if stale.is_empty() {
        println!("generated tests are up to date");
        ExitCode::SUCCESS
    } else {
        for line in &stale {
            eprintln!("{line}");
        }
        eprintln!("run `cargo run -p gen-corpus` to regenerate");
        ExitCode::FAILURE
    }
}

fn main() -> ExitCode {
    let mut do_check = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--check" => do_check = true,
            other => {
                eprintln!("usage: gen-corpus [--check] (unknown argument: {other})");
                return ExitCode::from(2);
            }
        }
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let corpus = root.join("test-corpus");
    let out = root.join("ops/tests/acceptance");

    let files = generate(&corpus);
    if do_check {
        check(&out, &files)
    } else {
        write(&out, &files)
    }
}
