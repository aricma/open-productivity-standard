//! Runs the language-agnostic corpus in `../../test-corpus`.
//!
//! - `valid/<case>/{claim.txt, given.<ext>}` — importing `given` must
//!   succeed.
//! - `invalid/<case>/{claim.txt, given.<ext>}` — importing `given` must
//!   fail; the spec does not prescribe a reason, so none is matched.
//! - `convert/<case>/{claim.txt, given.<ext>, expectation.<ext>}` — the two
//!   extensions are the transformation: `given` is read with the format
//!   named by its extension and written with the format named by the
//!   expectation's (same format = round-trip, another = conversion), the
//!   result is read back, and that model must equal the model of
//!   `expectation.<ext>`. Comparison is semantic: byte layout is not part
//!   of the standard.
//! - `roundtrips/<case>/{claim.txt, given.<ext>, expectation.<ext>}` — a
//!   same-format round-trip, run exactly like `convert` cases.
//!
//! Formats the crate does not implement are skipped and counted.

use ops_lib::Format;
use std::path::{Path, PathBuf};

/// `../../test-corpus`, resolved independently of the working directory.
const CORPUS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-corpus");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Group {
    Valid,
    Invalid,
    Convert,
    Roundtrips,
}

impl Group {
    fn from_name(name: &str) -> Option<Group> {
        Some(match name {
            "valid" => Group::Valid,
            "invalid" => Group::Invalid,
            "convert" => Group::Convert,
            "roundtrips" => Group::Roundtrips,
            _ => return None,
        })
    }
}

struct Case {
    name: String,
    group: Group,
    claim: String,
    given: PathBuf,
    given_format: Option<Format>,
    /// Only `convert/` and `roundtrips/` cases carry an expectation.
    expectation: Option<Expectation>,
}

struct Expectation {
    path: PathBuf,
    format: Option<Format>,
}

impl Case {
    fn support(&self) -> bool {
        self.given_format.is_some() && self.expectation.as_ref().is_none_or(|e| e.format.is_some())
    }
}

fn text(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
}

fn case_at(dir: &Path) -> Option<Case> {
    let mut claim = None;
    let mut given = None;
    let mut expectation = None;
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())) {
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

    let (Some(claim), Some(given)) = (claim, given) else {
        return None;
    };
    let relative = dir.strip_prefix(CORPUS).unwrap_or(dir);
    let group = relative
        .components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .and_then(Group::from_name)
        .unwrap_or_else(|| {
            panic!(
                "{}: not under valid/, invalid/, convert/, or roundtrips/",
                dir.display()
            )
        });
    let name = relative.display().to_string();
    let expectation = expectation.map(|path| Expectation {
        format: extension(&path).and_then(|e| Format::from_name(&e)),
        path,
    });
    Some(Case {
        name,
        group,
        claim: text(&claim).trim().to_string(),
        given_format: extension(&given).and_then(|e| Format::from_name(&e)),
        given,
        expectation,
    })
}

fn collect(dir: &Path, runnable: &mut Vec<Case>, skipped: &mut usize) {
    if let Some(case) = case_at(dir) {
        if case.support() {
            runnable.push(case);
        } else {
            *skipped += 1;
        }
    }
    let mut children: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .filter_map(|entry| {
            let path = entry.expect("dir entry").path();
            path.is_dir().then_some(path)
        })
        .collect();
    children.sort();
    for child in children {
        collect(&child, runnable, skipped);
    }
}

fn corpus() -> (Vec<Case>, usize) {
    let mut runnable = Vec::new();
    let mut skipped = 0;
    collect(Path::new(CORPUS), &mut runnable, &mut skipped);
    assert!(!runnable.is_empty(), "no runnable cases found in {CORPUS}");
    println!("{} runnable cases, {skipped} skipped", runnable.len());
    (runnable, skipped)
}

#[test]
fn convert_cases_roundtrip_semantically() {
    for case in corpus().0 {
        let Some(Expectation { path, format }) = &case.expectation else {
            assert!(
                !matches!(case.group, Group::Convert | Group::Roundtrips),
                "{}: needs an expectation.<ext>",
                case.name
            );
            continue;
        };
        let Some(given_format) = case.given_format else {
            continue;
        };
        let format = format.expect("output cases are supported");
        let given = text(&case.given);
        let expected = text(path);
        let tasks = ops_lib::read(given_format, &given)
            .unwrap_or_else(|e| panic!("{}: given rejected: {e}", case.name));
        let out = ops_lib::write(format, &tasks)
            .unwrap_or_else(|e| panic!("{}: write failed: {e}", case.name));
        let produced = ops_lib::read(format, &out)
            .unwrap_or_else(|e| panic!("{}: output rejected: {e}", case.name));
        let wanted = ops_lib::read(format, &expected)
            .unwrap_or_else(|e| panic!("{}: expectation rejected: {e}", case.name));
        assert_eq!(
            produced, wanted,
            "{}: {} (the exported model differs from the expectation)",
            case.name, case.claim
        );
    }
}

#[test]
fn valid_documents_are_accepted() {
    for case in corpus().0 {
        if case.group != Group::Valid {
            continue;
        }
        let given_format = case.given_format.expect("valid cases are supported");
        ops_lib::read(given_format, &text(&case.given))
            .unwrap_or_else(|e| panic!("{}: must be accepted — {}: {e}", case.name, case.claim));
    }
}

#[test]
fn invalid_documents_fail() {
    for case in corpus().0 {
        if case.group != Group::Invalid {
            continue;
        }
        let given_format = case.given_format.expect("invalid cases are supported");
        assert!(
            ops_lib::read(given_format, &text(&case.given)).is_err(),
            "{}: must be rejected — {}",
            case.name,
            case.claim
        );
    }
}
