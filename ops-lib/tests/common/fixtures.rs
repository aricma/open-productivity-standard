use super::round_trip_test_case::RoundTripTestCase;
use ops_lib::Format;
use std::path::{Path, PathBuf};

pub const OPS_TESTS: &str = "tests/fixtures/ops/tests";
pub const OPS_EXAMPLES: &str = "tests/fixtures/ops/examples";
pub const LOCAL: &str = "tests/fixtures/local";

pub fn read_fixture(dir: &str, filename: &str) -> String {
    std::fs::read_to_string(format!("{dir}/{filename}")).unwrap()
}

pub fn fixture_path_to_format(path: &Path) -> Format {
    match path.extension().and_then(|e| e.to_str()).unwrap() {
        "json" => Format::Json,
        "yaml" | "yml" => Format::Yaml,
        "jsonl" | "ndjson" => Format::Jsonl,
        other => panic!("unsupported fixture extension: {other}"),
    }
}

pub fn files_under(dir: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(files_under(&path.to_string_lossy()));
        } else {
            out.push(path);
        }
    }
    out
}

pub fn get_all_ops_fixture_files(kind: &str) -> Vec<PathBuf> {
    files_under(OPS_TESTS)
        .into_iter()
        .filter(|p| {
            p.parent()
                .and_then(|d| d.file_name())
                .is_some_and(|n| n == kind)
                && matches!(
                    p.extension().and_then(|e| e.to_str()),
                    Some("json" | "jsonl" | "yaml")
                )
        })
        .collect()
}

pub fn get_all_ops_example_files() -> Vec<PathBuf> {
    files_under(OPS_EXAMPLES)
        .into_iter()
        .filter(|p| p.file_name().is_some_and(|n| n != "README.md"))
        .filter(|p| {
            matches!(
                p.extension().and_then(|e| e.to_str()),
                Some("json" | "yaml" | "jsonl")
            )
        })
        .collect()
}

pub fn fixture_paths_to_roundtrips_test_cases(paths: Vec<PathBuf>) -> Vec<RoundTripTestCase> {
    paths
        .into_iter()
        .map(|path| RoundTripTestCase {
            message: path.file_name().unwrap().to_str().unwrap().to_string(),
            given: (
                fixture_path_to_format(&path),
                std::fs::read_to_string(&path).unwrap(),
            ),
        })
        .collect()
}
