//! The whole generated tree: where it lives, and every version, format, and
//! kind it holds.
//!
//! Planning is separate from writing: this says what the tree should contain,
//! and [`crate::output`] either writes it or checks it.

use crate::case::Group;
use crate::constants;
use crate::corpus::{self, directory_name, subdirectories};
use crate::emission;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// Every file to generate, as `(path relative to the output root, contents)`.
pub(crate) fn generated_files(root: &Path, out: &Path) -> Vec<(String, String)> {
    let versions = corpus_versions(root);
    let mut files = Vec::new();
    let mut main = String::from(constants::ACCEPTANCE_HEADER);
    for version in &versions {
        files.extend(version_files(root, out, version));
        writeln!(main, "mod {version};").unwrap();
    }
    files.insert(0, ("main.rs".to_string(), main));
    files
}

/// The versions under `specs/` that have a corpus, in order.
fn corpus_versions(root: &Path) -> Vec<String> {
    let specs = root.join(constants::SPECS_DIR);
    let mut versions: Vec<String> = subdirectories(&specs)
        .into_iter()
        .filter(|dir| dir.join(constants::TESTS_DIR).is_dir())
        .map(|dir| directory_name(&dir))
        .collect();
    versions.sort();
    assert!(
        !versions.is_empty(),
        "no spec version with a {} under {}",
        constants::TESTS_DIR,
        specs.display()
    );
    versions
}

/// Where the generated tests live, under the repository.
pub(crate) fn acceptance_dir(repository: &Path) -> PathBuf {
    repository.join(constants::ACCEPTANCE_DIR)
}

/// The repository, resolved from the build, so the tool works whatever the
/// working directory is.
pub(crate) fn repository_root() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::canonicalize(&root).unwrap_or(root)
}

/// One version's files: a module per format, one per kind inside it, plus the
/// cross-format `transform`, and the version module that lists them.
fn version_files(root: &Path, out: &Path, version: &str) -> Vec<(String, String)> {
    let cases = corpus::read_cases(root, version);
    let mut files = Vec::new();
    let mut modules: BTreeSet<String> = BTreeSet::new();

    let formats: BTreeSet<String> = cases
        .iter()
        .filter_map(|case| case.format.clone())
        .collect();
    for format in &formats {
        let mut format_module =
            emission::file_header(&format!("`specs/{version}/tests/{format}/` cases."));
        let mut kinds: BTreeSet<String> = BTreeSet::new();
        for group in Group::ALL {
            if group.crosses_formats() {
                continue;
            }
            let group_cases: Vec<&crate::case::Case> = cases
                .iter()
                .filter(|case| {
                    case.format.as_deref() == Some(format.as_str()) && case.group == group
                })
                .collect();
            if group_cases.is_empty() {
                continue;
            }
            let file = format!("{version}/{format}/{}.rs", group.dir());
            let summary = format!(
                "One `#[test]` per `specs/{version}/tests/{format}/{}/` case.",
                group.dir()
            );
            let mut content = emission::file_header(&summary);
            content.push_str(&emission::render_tests(root, out, &file, &group_cases));
            files.push((file, content));
            kinds.insert(group.dir().to_string());
        }
        for kind in &kinds {
            writeln!(format_module, "mod {kind};").unwrap();
        }
        files.push((format!("{version}/{format}/mod.rs"), format_module));
        modules.insert(format.clone());
    }

    let transforms: Vec<&crate::case::Case> = cases
        .iter()
        .filter(|case| case.group == Group::Transform)
        .collect();
    if !transforms.is_empty() {
        let file = format!("{version}/transform.rs");
        let summary = format!("One `#[test]` per `specs/{version}/tests/transform/` case.");
        let mut content = emission::file_header(&summary);
        content.push_str(&emission::render_tests(root, out, &file, &transforms));
        files.push((file, content));
        modules.insert(Group::Transform.dir().to_string());
    }

    let mut version_module =
        emission::file_header(&format!("The `{version}` corpus, one module per format."));
    for module in &modules {
        writeln!(version_module, "mod {module};").unwrap();
    }
    files.push((format!("{version}/mod.rs"), version_module));
    files
}
