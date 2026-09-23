//! Write tests: byte-stable output for valid models, and validation of
//! in-memory models (including forests that violate a format's
//! single-root rule) before serializing.

mod common;

use common::error_kind::ErrorKind;
use common::fixtures::{LOCAL, read_fixture};
use common::models::{example_model, full_model, meta};
use common::run_test_cases::{run_write_error_test_cases, run_write_test_cases};
use common::test_case::TestCase;
use ops_lib::{Format, Status, Task, ValidationError};
use serde_json::json;

/// Writing a valid model must reproduce its fixture character for
/// character — the writer is byte-stable per format.
#[test]
fn valid_models_write_exactly_their_fixture_characters() {
    let cases = [
        TestCase {
            message: "full model json matches its fixture".into(),
            given: (Format::Json, full_model()),
            expected: read_fixture(LOCAL, "valid/full-model.json"),
        },
        TestCase {
            message: "full model yaml matches its fixture".into(),
            given: (Format::Yaml, full_model()),
            expected: read_fixture(LOCAL, "valid/full-model.yaml"),
        },
        TestCase {
            message: "full model jsonl matches its fixture".into(),
            given: (Format::Jsonl, full_model()),
            expected: read_fixture(LOCAL, "valid/full-model.jsonl"),
        },
    ];

    run_write_test_cases(&cases);
}

/// In-memory models can be invalid; write must validate before
/// serializing. Each case carries its own format, since forests
/// are also rejected by the tree formats.
#[test]
fn invalid_models_are_rejected_before_writing() {
    let forest = vec![example_model(), Task::new("Personal".into(), Status::Open)];
    let cases = [
        TestCase {
            message: "duplicate ids across the forest are rejected".into(),
            given: (
                Format::Json,
                vec![
                    Task {
                        id: Some("x".into()),
                        title: "A".into(),
                        status: Status::Open,
                        ..Default::default()
                    },
                    Task {
                        id: Some("x".into()),
                        title: "B".into(),
                        status: Status::Open,
                        ..Default::default()
                    },
                ],
            ),
            expected: ErrorKind::Validation(ValidationError::DuplicateId("x".into())),
        },
        TestCase {
            message: "metadata key outside the charset is rejected".into(),
            given: (
                Format::Json,
                vec![Task {
                    title: "R".into(),
                    status: Status::Open,
                    metadata: Some(meta(json!({ "bad-key": true }))),
                    ..Default::default()
                }],
            ),
            expected: ErrorKind::Validation(ValidationError::InvalidMetadataKey("bad-key".into())),
        },
        TestCase {
            message: "version on a subtask is rejected".into(),
            given: (
                Format::Json,
                vec![Task {
                    title: "R".into(),
                    status: Status::Open,
                    subtasks: vec![Task {
                        title: "C".into(),
                        status: Status::Open,
                        version: Some("0".into()),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            ),
            expected: ErrorKind::Validation(ValidationError::VersionOnSubtask("C".into())),
        },
        TestCase {
            message: "unreleased version on a root is rejected".into(),
            given: (
                Format::Json,
                vec![Task {
                    title: "R".into(),
                    status: Status::Open,
                    version: Some("1".into()),
                    ..Default::default()
                }],
            ),
            expected: ErrorKind::Validation(ValidationError::UnknownOPSVersion("1".into())),
        },
        TestCase {
            message: "json holds exactly one root".into(),
            given: (Format::Json, forest.clone()),
            expected: ErrorKind::SingleRoot,
        },
        TestCase {
            message: "yaml holds exactly one root".into(),
            given: (Format::Yaml, forest.clone()),
            expected: ErrorKind::SingleRoot,
        },
    ];

    run_write_error_test_cases(&cases);
}
