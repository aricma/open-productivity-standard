//! Generates `ops/tests/acceptance/**` — one `#[test]` per case under
//! `specs/<version>/tests/` — so the suite is committed, reviewable, and
//! mirrors the corpus layout.
//!
//! The corpus is grouped by format, and each format by kind: `valid/`,
//! `invalid/`, `roundtrip/`, `rejects/`. `transform/` is the one group that
//! crosses formats. The test name is the case directory name.
//!
//! Formats the reference implementation does not claim become `#[ignore]`d
//! tests, so skips stay visible and counted.
//!
//! ```text
//! cargo run -p gen-corpus             regenerate, removing stale files
//! cargo run -p gen-corpus -- --check  fail if the committed files differ
//! ```
//!
//! The generator owns `ops/tests/acceptance/`: it writes every file it lists
//! and removes any other `.rs` in that directory.

use ops_lib::v0::Format;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// What a case asserts. Every group but `Transform` lives inside one
/// format's directory; `Transform` crosses formats and sits at the top.
///
/// `invalid/` runs both ways: a `given` in the enclosing format is rejected
/// by its reader, and a `given` in another format is a valid model that only
/// the enclosing format's writer refuses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Group {
    Valid,
    Invalid,
    Roundtrip,
    Transform,
}

impl Group {
    /// Every group, in the order its module is listed.
    const ALL: [Group; 4] = [
        Group::Valid,
        Group::Invalid,
        Group::Roundtrip,
        Group::Transform,
    ];

    fn dir(self) -> &'static str {
        match self {
            Group::Valid => "valid",
            Group::Invalid => "invalid",
            Group::Roundtrip => "roundtrip",
            Group::Transform => "transform",
        }
    }

    /// Whether the group sits at the corpus top instead of inside a format.
    fn crosses_formats(self) -> bool {
        matches!(self, Group::Transform)
    }

    fn needs_expectation(self) -> bool {
        matches!(self, Group::Roundtrip | Group::Transform)
    }
}

struct Case {
    version: String,
    /// The enclosing format directory; `None` for a `transform/` case.
    format: Option<String>,
    group: Group,
    name: String,
    claim: String,
    given_ext: String,
    /// Relative to the repo root, for `include_str!`.
    given: String,
    /// `(extension, path relative to the repo root)`.
    expectation: Option<(String, String)>,
}

impl Case {
    /// The `specs/…` path a message names.
    fn rel(&self) -> String {
        match &self.format {
            Some(format) => {
                format!(
                    "specs/{}/tests/{format}/{}/{}",
                    self.version,
                    self.group.dir(),
                    self.name
                )
            }
            None => format!(
                "specs/{}/tests/{}/{}",
                self.version,
                self.group.dir(),
                self.name
            ),
        }
    }

    /// Whether this case hands the enclosing format a model it cannot carry,
    /// so the failure is on write rather than on read. Inside a format
    /// directory, an `invalid` case whose `given` is in another format is
    /// that shape.
    fn rejects_the_writer(&self) -> bool {
        let (Some(format), Group::Invalid) = (&self.format, self.group) else {
            return false;
        };
        format_variant(&self.version, format) != format_variant(&self.version, &self.given_ext)
    }

    /// The `Format::…` path this case reads, and the one it writes (the
    /// same as the one it reads for a round-trip). `None` when the format is
    /// not claimed.
    fn calls(&self) -> (Option<String>, Option<String>) {
        let from = variant(&self.version, &self.given_ext);
        let to = if self.rejects_the_writer() {
            self.format
                .as_deref()
                .and_then(|format| variant(&self.version, format))
        } else {
            match self.group {
                Group::Roundtrip => from.clone(),
                Group::Transform => self
                    .expectation
                    .as_ref()
                    .and_then(|(ext, _)| variant(&self.version, ext)),
                Group::Valid | Group::Invalid => None,
            }
        };
        (from, to)
    }

    /// Every format name or extension this case exercises.
    fn formats(&self) -> Vec<&str> {
        let mut formats = vec![self.given_ext.as_str()];
        if let Some((ext, _)) = &self.expectation {
            formats.push(ext);
        }
        if self.rejects_the_writer() {
            formats.push(
                self.format
                    .as_deref()
                    .expect("a writer-reject case is inside a format directory"),
            );
        }
        formats
    }
}

fn extension(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or_else(|| panic!("{}: no extension", path.display()))
        .to_lowercase()
}

fn entries(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.is_dir())
        .collect();
    out.sort();
    out
}

fn files_of(dir: &Path) -> (PathBuf, PathBuf, Option<PathBuf>) {
    let mut claim = None;
    let mut given = None;
    let mut expectation = None;
    for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())) {
        let path = entry.expect("dir entry").path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let slot = if name == "claim.txt" {
            &mut claim
        } else if name.starts_with("given.") {
            &mut given
        } else if name.starts_with("expectation.") {
            &mut expectation
        } else {
            continue;
        };
        assert!(slot.is_none(), "{}: more than one {name}", dir.display());
        *slot = Some(path);
    }
    let name = dir.file_name().unwrap().to_string_lossy();
    (
        claim.unwrap_or_else(|| panic!("{name}: no claim.txt")),
        given.unwrap_or_else(|| panic!("{name}: no given.<ext>")),
        expectation,
    )
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or_else(|_| panic!("{} is not under {}", path.display(), root.display()))
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn case_at(root: &Path, dir: &Path, version: &str, format: Option<&str>, group: Group) -> Case {
    let (claim, given, expectation) = files_of(dir);
    let claim = fs::read_to_string(&claim)
        .unwrap_or_else(|e| panic!("read {}: {e}", claim.display()))
        .trim()
        .to_string();
    let case = Case {
        version: version.to_string(),
        format: format.map(str::to_string),
        group,
        name: dir.file_name().unwrap().to_string_lossy().into_owned(),
        claim,
        given_ext: extension(&given),
        given: relative(root, &given),
        expectation: expectation.map(|path| (extension(&path), relative(root, &path))),
    };
    assert!(
        !group.needs_expectation() || case.expectation.is_some(),
        "{}: needs an expectation.<ext>",
        case.rel()
    );
    case
}

/// Every case under one version's `tests/` directory.
fn cases_of_version(root: &Path, version: &str) -> Vec<Case> {
    let tests = root.join("specs").join(version).join("tests");
    let mut cases = Vec::new();
    for format_dir in entries(&tests) {
        let format = format_dir
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        if format == Group::Transform.dir() {
            for dir in entries(&format_dir) {
                cases.push(case_at(root, &dir, version, None, Group::Transform));
            }
            continue;
        }
        for group in Group::ALL {
            if group.crosses_formats() {
                continue;
            }
            let group_dir = format_dir.join(group.dir());
            if !group_dir.is_dir() {
                continue;
            }
            for dir in entries(&group_dir) {
                cases.push(case_at(root, &dir, version, Some(&format), group));
            }
        }
    }
    assert!(
        !cases.is_empty(),
        "{version}: no cases under {}",
        tests.display()
    );

    // Inside a format directory, `valid/` and `roundtrip/` read and write
    // that format, so their files must use its own extension. `invalid/`
    // may also hold a `given` in another format — a valid model the
    // enclosing writer refuses — so it is not constrained here; a typo
    // there makes the assertion fail instead.
    for case in &cases {
        let (Some(format), Group::Valid | Group::Roundtrip) = (&case.format, case.group) else {
            continue;
        };
        let enclosing = format_variant(version, format);
        let reads = case.given_ext.as_str();
        assert_eq!(
            enclosing,
            format_variant(version, reads),
            "{}: `given.{reads}` is not {format}",
            case.rel()
        );
        if let Some((ext, _)) = &case.expectation {
            assert_eq!(
                enclosing,
                format_variant(version, ext),
                "{}: `expectation.{ext}` is not {format}",
                case.rel()
            );
        }
    }
    cases
}

/// The `Format` variant the reference implementation uses for a corpus
/// format name or extension, or `None` when it does not claim that format.
fn format_variant(version: &str, format: &str) -> Option<String> {
    match version {
        "v0" => Format::from_name(format).map(|f| format!("{f:?}")),
        _ => None,
    }
}

/// The `Format::…` path for one format, version-prefixed.
fn variant(version: &str, format: &str) -> Option<String> {
    format_variant(version, format).map(|v| format!("ops_lib::{version}::Format::{v}"))
}

/// The `../..` prefix that gets from a generated file back to the repo root.
fn include_prefix(root: &Path, out: &Path, file: &str) -> String {
    let path = out.join(file);
    let parent = path.parent().expect("a generated file has a parent");
    assert!(
        parent.starts_with(root),
        "{} is not under {}",
        parent.display(),
        root.display()
    );
    let depth = parent
        .strip_prefix(root)
        .expect("checked above")
        .components()
        .count();
    "../".repeat(depth)
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

/// A case's file as an `include_str!` expression.
fn include(prefix: &str, path: &str) -> String {
    format!("include_str!(\"{prefix}{path}\")")
}

/// The test body for one case, or the name of a format the crate does not
/// claim, which makes the case `#[ignore]`d.
fn gen_body(root: &Path, out: &Path, file: &str, case: &Case) -> Result<String, String> {
    if let Some(missing) = case
        .formats()
        .into_iter()
        .find(|format| format_variant(&case.version, format).is_none())
    {
        return Err(missing.to_string());
    }
    let (from, to) = case.calls();
    let prefix = include_prefix(root, out, file);
    let api = format!("ops_lib::{}", case.version);
    Ok(body_of(
        case,
        &prefix,
        &api,
        &from.expect("checked"),
        to.as_deref(),
    ))
}

/// Renders one case's test body. Separate from [`gen_body`] so every shape
/// is exercised by a test, including the writer-reject one that no claimed
/// format reaches today.
fn body_of(case: &Case, prefix: &str, api: &str, from: &str, to: Option<&str>) -> String {
    let given = include(prefix, &case.given);
    let rel = case.rel();
    let claim = &case.claim;
    match case.group {
        Group::Valid => format!(
            r#"    let given = {given};
    {api}::read({from}, given)
        .unwrap_or_else(|e| panic!("{rel}: must be accepted — {claim}: {{e}}"));
"#
        ),
        Group::Invalid if case.rejects_the_writer() => {
            let to = to.expect("a writer-reject case has a target format");
            format!(
                r#"    let given = {given};
    let tasks = {api}::read({from}, given)
        .unwrap_or_else(|e| panic!("{rel}: given rejected: {{e}}"));
    assert!(
        {api}::write({to}, &tasks).is_err(),
        "{rel}: must be rejected — {claim}"
    );
"#
            )
        }
        Group::Invalid => format!(
            r#"    let given = {given};
    assert!(
        {api}::read({from}, given).is_err(),
        "{rel}: must be rejected — {claim}"
    );
"#
        ),
        Group::Roundtrip | Group::Transform => {
            let to = to.expect("a transform writes to a format");
            let expectation = case.expectation.as_ref().expect("checked when read");
            let expected = include(prefix, &expectation.1);
            format!(
                r#"    let given = {given};
    let expected = {expected};
    let tasks = {api}::read({from}, given)
        .unwrap_or_else(|e| panic!("{rel}: given rejected: {{e}}"));
    let out = {api}::write({to}, &tasks)
        .unwrap_or_else(|e| panic!("{rel}: write failed: {{e}}"));
    let produced = {api}::read({to}, &out)
        .unwrap_or_else(|e| panic!("{rel}: output rejected: {{e}}"));
    let wanted = {api}::read({to}, expected)
        .unwrap_or_else(|e| panic!("{rel}: expectation rejected: {{e}}"));
    assert_eq!(
        produced, wanted,
        "{rel}: {claim} (the exported model differs from the expectation)"
    );
"#
            )
        }
    }
}

/// Emits the `#[test]` blocks for a group of cases, ending with a trailing
/// newline.
fn emit_tests(root: &Path, out: &Path, file: &str, cases: &[&Case]) -> String {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut running = String::new();
    let mut ignored = Vec::new();
    for case in cases {
        let base = slug(&case.name);
        let n = seen.entry(base.clone()).or_insert(0);
        *n += 1;
        let name = if *n == 1 { base } else { format!("{base}_{n}") };
        match gen_body(root, out, file, case) {
            Ok(body) => {
                writeln!(
                    running,
                    "#[rustfmt::skip]\n#[test]\nfn {name}() {{\n{body}}}\n"
                )
                .unwrap();
            }
            Err(format) => ignored.push((name, format)),
        }
    }
    for (name, format) in ignored {
        writeln!(
            running,
            "#[rustfmt::skip]\n#[test]\n#[ignore = \"unsupported format: {format}\"]\nfn {name}() {{}}\n"
        )
        .unwrap();
    }
    if !running.is_empty() {
        running.pop();
    }
    running
}

fn header(summary: &str) -> String {
    format!("//! {summary} Generated by `gen-corpus`; do not edit by hand.\n\n")
}

fn generate(root: &Path, out: &Path) -> Vec<(String, String)> {
    let specs = root.join("specs");
    let mut versions: Vec<String> = entries(&specs)
        .into_iter()
        .filter(|dir| dir.join("tests").is_dir())
        .map(|dir| dir.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    versions.sort();
    assert!(
        !versions.is_empty(),
        "no spec version with a tests/ under {}",
        specs.display()
    );

    let mut main = String::from(
        "//! Runs the language-agnostic corpus in `specs/<version>/tests`, one\n\
         //! `#[test]` per case, grouped by format. Cases for a format this crate\n\
         //! does not claim are `#[ignore]`d, so skips stay visible and counted.\n\
         //!\n\
         //! Generated by `gen-corpus`; do not edit by hand.\n\n",
    );
    let mut files: Vec<(String, String)> = Vec::new();

    for version in &versions {
        let cases = cases_of_version(root, version);
        let mut version_mod = header(&format!("The `{version}` corpus, one module per format."));
        let mut modules: BTreeSet<String> = BTreeSet::new();

        let formats: BTreeSet<String> = cases.iter().filter_map(|c| c.format.clone()).collect();
        for format in &formats {
            let mut format_mod = header(&format!("`specs/{version}/tests/{format}/` cases."));
            let mut groups: BTreeSet<String> = BTreeSet::new();
            for group in Group::ALL {
                if group.crosses_formats() {
                    continue;
                }
                let group_cases: Vec<&Case> = cases
                    .iter()
                    .filter(|c| c.format.as_deref() == Some(format.as_str()) && c.group == group)
                    .collect();
                if group_cases.is_empty() {
                    continue;
                }
                let file = format!("{version}/{format}/{}.rs", group.dir());
                let summary = format!(
                    "One `#[test]` per `specs/{version}/tests/{format}/{}/` case.",
                    group.dir()
                );
                let mut content = header(&summary);
                content.push_str(&emit_tests(root, out, &file, &group_cases));
                files.push((file, content));
                groups.insert(group.dir().to_string());
            }
            for group in &groups {
                writeln!(format_mod, "mod {group};").unwrap();
            }
            files.push((format!("{version}/{format}/mod.rs"), format_mod));
            modules.insert(format.clone());
        }

        let transform: Vec<&Case> = cases
            .iter()
            .filter(|c| c.group == Group::Transform)
            .collect();
        if !transform.is_empty() {
            let file = format!("{version}/transform.rs");
            let mut content = header(&format!(
                "One `#[test]` per `specs/{version}/tests/transform/` case."
            ));
            content.push_str(&emit_tests(root, out, &file, &transform));
            files.push((file, content));
            modules.insert(Group::Transform.dir().to_string());
        }

        for module in &modules {
            writeln!(version_mod, "mod {module};").unwrap();
        }
        files.push((format!("{version}/mod.rs"), version_mod));
        writeln!(main, "mod {version};").unwrap();
    }

    files.insert(0, ("main.rs".to_string(), main));
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
    let root = fs::canonicalize(&root).unwrap_or(root);
    let out = root.join("ops/tests/acceptance");

    let files = generate(&root, &out);
    if do_check {
        check(&out, &files)
    } else {
        write(&out, &files)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    fn case(group: Group, format: Option<&str>, given: &str, expectation: Option<&str>) -> Case {
        Case {
            version: "v0".to_string(),
            format: format.map(str::to_string),
            group,
            name: "case".to_string(),
            claim: "the claim".to_string(),
            given_ext: given.to_string(),
            given: format!("specs/v0/tests/place/given.{given}"),
            expectation: expectation.map(|ext| {
                (
                    ext.to_string(),
                    format!("specs/v0/tests/place/expectation.{ext}"),
                )
            }),
        }
    }

    /// Rendered without the support gate, so every shape is covered —
    /// including the writer-reject one no claimed format reaches today.
    fn body(case: &Case, to: Option<&str>) -> String {
        let from = format!("ops_lib::v0::Format::{:?}", case.given_ext);
        let to = to.map(str::to_string);
        body_of(case, "../", "ops_lib::v0", &from, to.as_deref())
    }

    #[test]
    fn each_shape_generates_its_assertion() {
        let valid = body(&case(Group::Valid, Some("json"), "json", None), None);
        assert!(
            valid.contains("ops_lib::v0::read(ops_lib::v0::Format::\"json\""),
            "{valid}"
        );
        assert!(valid.contains("must be accepted"), "{valid}");

        // An `invalid` case whose given is in the enclosing format is a
        // reader rejection.
        let invalid = body(&case(Group::Invalid, Some("json"), "json", None), None);
        assert!(invalid.contains(".is_err()"), "{invalid}");
        assert!(
            invalid.contains("ops_lib::v0::read(ops_lib::v0::Format::\"json\""),
            "{invalid}"
        );

        // One whose given is in another format is a writer rejection.
        let refused = body(
            &case(Group::Invalid, Some("markdown"), "json", None),
            Some("ops_lib::v0::Format::Markdown"),
        );
        assert!(
            refused.contains("ops_lib::v0::read(ops_lib::v0::Format::\"json\""),
            "{refused}"
        );
        assert!(
            refused.contains("ops_lib::v0::write(ops_lib::v0::Format::Markdown"),
            "{refused}"
        );
        assert!(refused.contains(".is_err()"), "{refused}");

        let roundtrip = body(
            &case(Group::Roundtrip, Some("json"), "json", Some("json")),
            Some("ops_lib::v0::Format::Json"),
        );
        assert!(
            roundtrip.contains("ops_lib::v0::write(ops_lib::v0::Format::Json"),
            "{roundtrip}"
        );

        let transform = body(
            &case(Group::Transform, None, "json", Some("yaml")),
            Some("ops_lib::v0::Format::Yaml"),
        );
        assert!(
            transform.contains("ops_lib::v0::write(ops_lib::v0::Format::Yaml"),
            "{transform}"
        );
    }

    /// Which direction an `invalid` case runs: the enclosing format's reader,
    /// or — when the given is in another format — its writer.
    #[test]
    fn invalid_runs_the_way_the_given_says() {
        assert!(!case(Group::Invalid, Some("json"), "json", None).rejects_the_writer());
        assert!(case(Group::Invalid, Some("markdown"), "json", None).rejects_the_writer());
        assert!(!case(Group::Valid, Some("markdown"), "md", None).rejects_the_writer());
    }

    /// A format the crate does not claim makes a case ignored instead of
    /// generating a body. The name in the reason is whichever the case
    /// mentions first.
    #[test]
    fn an_unsupported_format_is_ignored() {
        let root = Path::new("/repo");
        let out = root.join("ops/tests/acceptance");
        let markdown = case(Group::Valid, Some("markdown"), "md", None);
        assert_eq!(
            gen_body(root, &out, "v0/markdown/valid.rs", &markdown).unwrap_err(),
            "md"
        );
        let refused = case(Group::Invalid, Some("markdown"), "json", None);
        assert_eq!(
            gen_body(root, &out, "v0/markdown/invalid.rs", &refused).unwrap_err(),
            "markdown"
        );
    }

    #[test]
    fn test_names_are_slugged() {
        assert_eq!(
            slug("A title is one non-empty line."),
            "a_title_is_one_non_empty_line"
        );
    }
}
