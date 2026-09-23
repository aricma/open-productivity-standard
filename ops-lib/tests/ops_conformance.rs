//! OPS conformance, at the level of rough guidelines rather than exact
//! error classes: every example must survive the round-trip, every
//! `valid/` fixture must round-trip without failing, every `invalid/`
//! fixture must be rejected. Fixtures: `tests/fixtures/ops/{examples,tests}`,
//! the standard's own documents, read in place so they cannot drift.

mod common;

use common::fixtures::{
    fixture_paths_to_roundtrips_test_cases, get_all_ops_example_files, get_all_ops_fixture_files,
};
use common::run_test_cases::{run_roundtrip_fails, run_roundtrip_passes};

#[test]
fn all_examples_roundtrip_to_an_identical_document() {
    let cases = fixture_paths_to_roundtrips_test_cases(get_all_ops_example_files());
    assert_eq!(
        cases.len(),
        6,
        "all supported example documents must be found"
    );
    run_roundtrip_passes(&cases);
}

#[test]
fn all_valid_fixtures_roundtrip_without_failing() {
    let cases = fixture_paths_to_roundtrips_test_cases(get_all_ops_fixture_files("valid"));
    assert_eq!(cases.len(), 25, "all valid fixtures must be found");
    run_roundtrip_passes(&cases);
}

#[test]
fn all_invalid_fixtures_are_rejected_by_the_roundtrip() {
    let cases = fixture_paths_to_roundtrips_test_cases(get_all_ops_fixture_files("invalid"));
    assert_eq!(cases.len(), 23, "all invalid fixtures must be found");
    run_roundtrip_fails(&cases);
}
