//! Table-driven read tests for our own `local/` fixtures: a few valid
//! cases pinning the full model per format, many small invalid cases
//! pinning every error across formats and nested/flat forms. The
//! OPS-sourced cases live in `ops_conformance.rs`.

mod common;

use common::error_kind::ErrorKind;
use common::expectation::Expectation;
use common::fixtures::{LOCAL, read_fixture};
use common::models::full_model;
use common::run_test_cases::run_read_test_cases;
use common::test_case::TestCase;
use ops_lib::{Format, ValidationError};

/// A handful of valid documents, each carrying the entire possible Task
/// model (all fields, two nesting levels) in one of the supported
/// formats.
#[test]
fn valid_reads_expect_models() {
    let cases = [
        TestCase {
            message: "full task model in json".into(),
            given: (Format::Json, read_fixture(LOCAL, "valid/full-model.json")),
            expected: Expectation::Model(vec![full_model()]),
        },
        TestCase {
            message: "full task model in yaml".into(),
            given: (Format::Yaml, read_fixture(LOCAL, "valid/full-model.yaml")),
            expected: Expectation::Model(vec![full_model()]),
        },
        TestCase {
            message: "full task model in jsonl (flat form)".into(),
            given: (Format::Jsonl, read_fixture(LOCAL, "valid/full-model.jsonl")),
            expected: Expectation::Model(vec![full_model()]),
        },
    ];

    run_read_test_cases(&cases);
}

/// Many small invalid documents, one per (error, format, nested/flat)
/// combination — each breaks exactly one rule.
#[test]
fn invalid_reads_expect_errors() {
    let cases = [
        TestCase {
            message: "metadata key `bad-key` breaks the charset in json".into(),
            given: (
                Format::Json,
                read_fixture(LOCAL, "invalid/metadata-key.json"),
            ),
            expected: Expectation::Error(ErrorKind::Validation(
                ValidationError::InvalidMetadataKey("bad-key".into()),
            )),
        },
        TestCase {
            message: "metadata key `bad-key` breaks the charset in yaml".into(),
            given: (
                Format::Yaml,
                read_fixture(LOCAL, "invalid/metadata-key.yaml"),
            ),
            expected: Expectation::Error(ErrorKind::Validation(
                ValidationError::InvalidMetadataKey("bad-key".into()),
            )),
        },
        TestCase {
            message: "metadata key `bad-key` breaks the charset in jsonl".into(),
            given: (
                Format::Jsonl,
                read_fixture(LOCAL, "invalid/metadata-key.jsonl"),
            ),
            expected: Expectation::Error(ErrorKind::Validation(
                ValidationError::InvalidMetadataKey("bad-key".into()),
            )),
        },
        TestCase {
            message: "duplicate id `x` in json".into(),
            given: (
                Format::Json,
                read_fixture(LOCAL, "invalid/duplicate-id.json"),
            ),
            expected: Expectation::Error(ErrorKind::Validation(ValidationError::DuplicateId(
                "x".into(),
            ))),
        },
        TestCase {
            message: "duplicate id `x` in yaml".into(),
            given: (
                Format::Yaml,
                read_fixture(LOCAL, "invalid/duplicate-id.yaml"),
            ),
            expected: Expectation::Error(ErrorKind::Validation(ValidationError::DuplicateId(
                "x".into(),
            ))),
        },
        TestCase {
            message: "duplicate id `x` in jsonl".into(),
            given: (
                Format::Jsonl,
                read_fixture(LOCAL, "invalid/duplicate-id.jsonl"),
            ),
            expected: Expectation::Error(ErrorKind::Validation(ValidationError::DuplicateId(
                "x".into(),
            ))),
        },
        TestCase {
            message: "version `0` on a subtask in json".into(),
            given: (
                Format::Json,
                read_fixture(LOCAL, "invalid/version-on-subtask.json"),
            ),
            expected: Expectation::Error(ErrorKind::Validation(ValidationError::VersionOnSubtask(
                "Child".into(),
            ))),
        },
        TestCase {
            message: "version `0` on a subtask in yaml".into(),
            given: (
                Format::Yaml,
                read_fixture(LOCAL, "invalid/version-on-subtask.yaml"),
            ),
            expected: Expectation::Error(ErrorKind::Validation(ValidationError::VersionOnSubtask(
                "Child".into(),
            ))),
        },
        TestCase {
            message: "version `0` on a subtask in jsonl".into(),
            given: (
                Format::Jsonl,
                read_fixture(LOCAL, "invalid/version-on-subtask.jsonl"),
            ),
            expected: Expectation::Error(ErrorKind::Validation(ValidationError::VersionOnSubtask(
                "Child".into(),
            ))),
        },
        TestCase {
            message: "unreleased ops version `1` in json".into(),
            given: (
                Format::Json,
                read_fixture(LOCAL, "invalid/unknown-version.json"),
            ),
            expected: Expectation::Error(ErrorKind::Validation(
                ValidationError::UnknownOPSVersion("1".into()),
            )),
        },
        TestCase {
            message: "unreleased ops version `1` in yaml".into(),
            given: (
                Format::Yaml,
                read_fixture(LOCAL, "invalid/unknown-version.yaml"),
            ),
            expected: Expectation::Error(ErrorKind::Validation(
                ValidationError::UnknownOPSVersion("1".into()),
            )),
        },
        TestCase {
            message: "unreleased ops version `1` in jsonl".into(),
            given: (
                Format::Jsonl,
                read_fixture(LOCAL, "invalid/unknown-version.jsonl"),
            ),
            expected: Expectation::Error(ErrorKind::Validation(
                ValidationError::UnknownOPSVersion("1".into()),
            )),
        },
        TestCase {
            message: "flat record references missing id `missing` in jsonl".into(),
            given: (
                Format::Jsonl,
                read_fixture(LOCAL, "invalid/missing-reference.jsonl"),
            ),
            expected: Expectation::Error(ErrorKind::Validation(ValidationError::MissingReference(
                "missing".into(),
            ))),
        },
        TestCase {
            message: "subtask `x` has more than one parent in jsonl".into(),
            given: (
                Format::Jsonl,
                read_fixture(LOCAL, "invalid/shared-child.jsonl"),
            ),
            expected: Expectation::Error(ErrorKind::Validation(ValidationError::SharedChild(
                "x".into(),
            ))),
        },
        TestCase {
            message: "references form a cycle with no root in jsonl".into(),
            given: (
                Format::Jsonl,
                read_fixture(LOCAL, "invalid/cyclic-reference.jsonl"),
            ),
            expected: Expectation::Error(ErrorKind::Validation(ValidationError::CyclicReference)),
        },
        TestCase {
            message: "malformed json".into(),
            given: (Format::Json, read_fixture(LOCAL, "invalid/malformed.json")),
            expected: Expectation::Error(ErrorKind::Json),
        },
        TestCase {
            message: "malformed yaml".into(),
            given: (Format::Yaml, read_fixture(LOCAL, "invalid/malformed.yaml")),
            expected: Expectation::Error(ErrorKind::Yaml),
        },
        TestCase {
            message: "malformed jsonl line".into(),
            given: (
                Format::Jsonl,
                read_fixture(LOCAL, "invalid/malformed.jsonl"),
            ),
            expected: Expectation::Error(ErrorKind::Json),
        },
    ];

    run_read_test_cases(&cases);
}
