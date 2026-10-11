//! Reading the corpus from disk, and refusing to read it wrongly.
//!
//! Everything that can be wrong about a corpus directory is checked here, and
//! checked loudly. A case filed in the wrong place used to be walked past in
//! silence, which is how four cases went missing once.

use crate::case::{Case, Group};
use crate::constants;
use crate::formats::Implementation;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Every case under one version's `tests/` directory.
pub(crate) fn read_cases(root: &Path, version: &str, implementation: &Implementation) -> Vec<Case> {
    let tests = root
        .join(constants::SPECS_DIR)
        .join(version)
        .join(constants::TESTS_DIR);
    let formats = documented_formats(root, version);
    let kinds = format_kinds();
    let mut cases = Vec::new();

    for format_dir in subdirectories(&tests) {
        let format = directory_name(&format_dir);
        if format == Group::Transform.dir() {
            for dir in subdirectories(&format_dir) {
                cases.push(read_case(root, &dir, version, None, Group::Transform));
            }
            continue;
        }

        check_format_is_documented(root, &format_dir, &formats);
        for group in Group::ALL {
            if group.crosses_formats() {
                continue;
            }
            let group_dir = format_dir.join(group.dir());
            if !group_dir.is_dir() {
                continue;
            }
            for dir in subdirectories(&group_dir) {
                cases.push(read_case(root, &dir, version, Some(&format), group));
            }
        }
        check_only_known_kinds(root, &format_dir, &kinds);
    }

    assert!(
        !cases.is_empty(),
        "{version}: no cases under {}",
        tests.display()
    );
    for case in &cases {
        check_case_formats(case, &formats, implementation);
    }
    cases
}

/// One case directory, read.
fn read_case(root: &Path, dir: &Path, version: &str, format: Option<&str>, group: Group) -> Case {
    let files = case_files(dir);
    let name = directory_name(dir);
    let claim = fs::read_to_string(&files.claim)
        .unwrap_or_else(|e| panic!("read {}: {e}", files.claim.display()))
        .trim()
        .to_string();
    let expectation = files
        .expectation
        .map(|path| (file_extension(&path), path_from_root(root, &path)));

    // What the case writes: a conversion names its target in its directory
    // name, and a round-trip writes its own format, which is the expectation's.
    let target_ext = match (group, &expectation) {
        (Group::Transform, _) => Some(transform_direction(&name).1),
        (Group::Roundtrip, Some((ext, _))) => Some(ext.clone()),
        _ => None,
    };
    // A skip report names the format the way the corpus does — the folder, or
    // the name a conversion's directory spells — so the same format is not
    // reported as `md` in one case and `markdown` in another.
    let names = match group {
        Group::Transform => {
            let (from, to) = transform_direction(&name);
            vec![from, to]
        }
        Group::Valid | Group::Invalid | Group::Roundtrip => vec![
            format
                .expect("a case under a format folder names its format")
                .to_string(),
        ],
    };

    let case = Case {
        version: version.to_string(),
        format: format.map(str::to_string),
        group,
        name,
        claim,
        given_ext: file_extension(&files.given),
        given: path_from_root(root, &files.given),
        target_ext,
        names,
        expectation,
    };

    // Which kinds carry an expectation: a round-trip always, a conversion only
    // when it can be done, nothing else ever.
    match (group, &case.expectation) {
        (Group::Roundtrip, None) => {
            panic!(
                "{}: needs an {}<ext>",
                case.corpus_path(),
                constants::EXPECTATION_PREFIX
            )
        }
        (Group::Valid | Group::Invalid, Some(_)) => panic!(
            "{}: only a roundtrip or a transform carries an {}<ext>",
            case.corpus_path(),
            constants::EXPECTATION_PREFIX
        ),
        _ => {}
    }
    case
}

/// A conversion's directory name, split into the formats it converts between:
/// `<from>-to-<to>`, with anything after the target describing the case.
fn transform_direction(case_dir_name: &str) -> (String, String) {
    let parts: Vec<&str> = case_dir_name.split('-').collect();
    assert!(
        parts.len() >= 3 && parts[1] == "to",
        "{case_dir_name}: a conversion is named `<from>-to-<to>`"
    );
    (parts[0].to_string(), parts[2].to_string())
}

/// A case directory's files.
struct CaseFiles {
    claim: PathBuf,
    given: PathBuf,
    expectation: Option<PathBuf>,
}

/// A case directory's three files, with the two it must have.
fn case_files(dir: &Path) -> CaseFiles {
    let mut claim = None;
    let mut given = None;
    let mut expectation = None;
    for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())) {
        let path = entry.expect("dir entry").path();
        let name = directory_name(&path);
        let slot = if name == constants::CLAIM_FILE {
            &mut claim
        } else if name.starts_with(constants::GIVEN_PREFIX) {
            &mut given
        } else if name.starts_with(constants::EXPECTATION_PREFIX) {
            &mut expectation
        } else {
            continue;
        };
        assert!(slot.is_none(), "{}: more than one {name}", dir.display());
        *slot = Some(path);
    }
    let case = directory_name(dir);
    CaseFiles {
        claim: claim.unwrap_or_else(|| panic!("{case}: no {}", constants::CLAIM_FILE)),
        given: given.unwrap_or_else(|| panic!("{case}: no {}<ext>", constants::GIVEN_PREFIX)),
        expectation,
    }
}

/// The formats a version documents, by directory name: one `formats/<name>.md`
/// each. The corpus is laid out by these names, so a case filed under a name
/// that is not one is a mistake rather than a format nobody implements.
fn documented_formats(root: &Path, version: &str) -> BTreeSet<String> {
    let dir = root
        .join(constants::SPECS_DIR)
        .join(version)
        .join(constants::FORMATS_DIR);
    let mut names = BTreeSet::new();
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())) {
        let path = entry.expect("dir entry").path();
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if path.extension().and_then(|e| e.to_str()) == Some("md") && name != "README" {
            names.insert(name);
        }
    }
    assert!(
        !names.is_empty(),
        "no formats documented in {}",
        dir.display()
    );
    names
}

/// The kinds a format directory may hold.
fn format_kinds() -> Vec<&'static str> {
    Group::ALL
        .iter()
        .filter(|group| !group.crosses_formats())
        .map(|group| group.dir())
        .collect()
}

/// Fails on a directory in `tests/` that is not a format the version documents.
fn check_format_is_documented(root: &Path, format_dir: &Path, formats: &BTreeSet<String>) {
    let format = directory_name(format_dir);
    assert!(
        formats.contains(&format),
        "{} is not a documented format; expected one of: {}",
        path_from_root(root, format_dir),
        formats.iter().cloned().collect::<Vec<_>>().join(", ")
    );
}

/// Fails on a directory inside a format that is not one of the corpus's kinds —
/// the other way a case gets filed where nothing will read it.
fn check_only_known_kinds(root: &Path, format_dir: &Path, kinds: &[&str]) {
    for stray in subdirectories(format_dir) {
        let kind = directory_name(&stray);
        assert!(
            kinds.contains(&kind.as_str()),
            "{} is not one of this corpus's kinds: {}",
            path_from_root(root, &stray),
            kinds.join(", ")
        );
    }
}

/// Fails unless a case's files are the formats the case says it is about: the
/// format it is filed under, or — for a conversion, which is filed at the top —
/// the two its name names.
fn check_case_formats(case: &Case, formats: &BTreeSet<String>, implementation: &Implementation) {
    let Some(format) = &case.format else {
        let (from, to) = transform_direction(&case.name);
        let given_file = format!("{}{}", constants::GIVEN_PREFIX, case.given_ext);
        for named in [&from, &to] {
            assert!(
                formats.contains(named),
                "{}: `{named}` is not a documented format",
                case.corpus_path()
            );
        }
        assert!(
            implementation.same(&from, &case.given_ext),
            "{}: `{given_file}` is not {from}",
            case.corpus_path()
        );
        if let Some((ext, _)) = &case.expectation {
            let expectation_file = format!("{}{ext}", constants::EXPECTATION_PREFIX);
            assert!(
                implementation.same(&to, ext),
                "{}: `{expectation_file}` is not {to}",
                case.corpus_path()
            );
        }
        return;
    };

    let given_file = format!("{}{}", constants::GIVEN_PREFIX, case.given_ext);
    assert!(
        implementation.same(format, &case.given_ext),
        "{}: `{given_file}` is not {format}",
        case.corpus_path()
    );
    if let Some((ext, _)) = &case.expectation {
        let expectation_file = format!("{}{ext}", constants::EXPECTATION_PREFIX);
        assert!(
            implementation.same(format, ext),
            "{}: `{expectation_file}` is not {format}",
            case.corpus_path()
        );
    }
}

/// The subdirectories of `dir`, sorted by name. Shared with [`crate::plan`],
/// which walks `specs/` the same way.
pub(crate) fn subdirectories(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.is_dir())
        .collect();
    out.sort();
    out
}

/// A path's last component. Shared with [`crate::plan`], which names the
/// versions the same way.
pub(crate) fn directory_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// `path` relative to `root`, `/`-separated, for `include_str!`.
fn path_from_root(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or_else(|_| panic!("{} is not under {}", path.display(), root.display()))
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// A file's extension, lowercased.
fn file_extension(path: &Path) -> String {
    path.extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_else(|| panic!("{}: no extension", path.display()))
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    /// What the repository's implementation declares.
    fn implementation() -> Implementation {
        Implementation::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ops/formats.txt"))
    }

    /// A scratch version tree: one documented format, plus whatever else the
    /// test files. Returns the root the generator would be pointed at.
    fn scratch(name: &str, files: &[(&str, &str)]) -> PathBuf {
        let root = std::env::temp_dir().join(format!("gen-corpus-{name}"));
        let _ = fs::remove_dir_all(&root);
        for (rel, content) in files {
            let path = root.join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, content).unwrap();
        }
        root
    }

    /// The formats the conversion tests below convert between.
    const CONVERSION: &[(&str, &str)] = &[
        ("specs/v0/formats/json.md", "# JSON\n"),
        ("specs/v0/formats/jsonl.md", "# JSONL\n"),
        ("specs/v0/formats/yaml.md", "# YAML\n"),
    ];

    /// A case filed under a name that is not a documented format used to be
    /// walked past in silence, which is how four cases went missing once.
    #[test]
    #[should_panic(expected = "is not a documented format")]
    fn an_undocumented_format_is_not_ignored() {
        let root = scratch(
            "undocumented",
            &[
                ("specs/v0/formats/json.md", "# JSON\n"),
                ("specs/v0/tests/json/valid/ok/claim.txt", "accepted."),
                ("specs/v0/tests/json/valid/ok/given.json", "{}"),
                ("specs/v0/tests/invalid/json/ok/claim.txt", "filed wrongly."),
                ("specs/v0/tests/invalid/json/ok/given.json", "{}"),
            ],
        );
        read_cases(&root, "v0", &implementation());
    }

    /// The same for a case in the right format but the wrong kind.
    #[test]
    #[should_panic(expected = "is not one of this corpus's kinds")]
    fn an_unknown_kind_inside_a_format_is_not_ignored() {
        let root = scratch(
            "kind",
            &[
                ("specs/v0/formats/json.md", "# JSON\n"),
                ("specs/v0/tests/json/valid/ok/claim.txt", "accepted."),
                ("specs/v0/tests/json/valid/ok/given.json", "{}"),
                ("specs/v0/tests/json/roundtrips/ok/claim.txt", "wrong kind."),
                ("specs/v0/tests/json/roundtrips/ok/given.json", "{}"),
            ],
        );
        read_cases(&root, "v0", &implementation());
    }

    /// An `invalid` case is a document its own format's reader rejects, so a
    /// document in another format is a mistake.
    #[test]
    #[should_panic(expected = "is not json")]
    fn an_invalid_case_must_use_its_format() {
        let root = scratch(
            "mismatch",
            &[
                ("specs/v0/formats/json.md", "# JSON\n"),
                ("specs/v0/formats/yaml.md", "# YAML\n"),
                ("specs/v0/tests/json/invalid/ok/claim.txt", "rejected."),
                ("specs/v0/tests/json/invalid/ok/given.yaml", "title: t\n"),
            ],
        );
        read_cases(&root, "v0", &implementation());
    }

    /// A conversion's name says which formats it converts between, and that has
    /// to agree with the document it reads.
    #[test]
    #[should_panic(expected = "is not yaml")]
    fn a_conversion_name_must_agree_with_its_given() {
        let mut files = CONVERSION.to_vec();
        files.push((
            "specs/v0/tests/transform/yaml-to-jsonl/claim.txt",
            "converts.",
        ));
        files.push(("specs/v0/tests/transform/yaml-to-jsonl/given.json", "{}"));
        let root = scratch("direction", &files);
        read_cases(&root, "v0", &implementation());
    }

    /// A conversion with nothing to compare against is a conversion that must
    /// fail, and it still has to name a target.
    #[test]
    fn a_conversion_without_an_expectation_names_its_target() {
        let mut files = CONVERSION.to_vec();
        files.push((
            "specs/v0/tests/transform/json-to-jsonl/claim.txt",
            "cannot.",
        ));
        files.push(("specs/v0/tests/transform/json-to-jsonl/given.json", "{}"));
        let root = scratch("target", &files);
        let cases = read_cases(&root, "v0", &implementation());
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].target_ext.as_deref(), Some("jsonl"));
        assert!(cases[0].must_fail());
    }

    /// A malformed conversion name is a mistake, not a case nobody can run.
    #[test]
    #[should_panic(expected = "a conversion is named")]
    fn a_malformed_conversion_name_is_not_ignored() {
        let mut files = CONVERSION.to_vec();
        files.push(("specs/v0/tests/transform/json-tojsonl/claim.txt", "cannot."));
        files.push(("specs/v0/tests/transform/json-tojsonl/given.json", "{}"));
        let root = scratch("malformed", &files);
        read_cases(&root, "v0", &implementation());
    }
}
