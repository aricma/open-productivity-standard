//! Model-level tests: cases that cannot be expressed as a document in the
//! portable corpus — building an invalid in-memory model, the full-model
//! round-trip, and writer idempotence.

use ops_lib::v0::{Format, Status, Task};
use serde_json::{Map, Value, json};

fn meta(value: Value) -> Map<String, Value> {
    value.as_object().unwrap().clone()
}

/// Every field populated, two nesting levels.
fn full_model() -> Task {
    Task {
        title: "Full task model".into(),
        status: Status::Open,
        version: Some("0".into()),
        id: Some("root".into()),
        notes: Some("Root notes".into()),
        metadata: Some(meta(json!({
            "priority": "high",
            "tags": ["infra", "auth"],
            "count": 3,
            "active": true,
            "nested": { "key": "value" },
        }))),
        subtasks: vec![
            Task {
                id: Some("child-1".into()),
                title: "Child one".into(),
                status: Status::Done,
                notes: Some("Child notes".into()),
                metadata: Some(meta(json!({ "estimate": 2 }))),
                subtasks: vec![Task {
                    id: Some("grandchild".into()),
                    title: "Grandchild".into(),
                    status: Status::Open,
                    ..Default::default()
                }],
                ..Default::default()
            },
            Task {
                id: Some("child-2".into()),
                title: "Child two".into(),
                status: Status::Open,
                ..Default::default()
            },
        ],
    }
}

#[test]
fn full_model_roundtrips_through_every_format() {
    for format in [Format::Json, Format::Yaml, Format::Jsonl] {
        let out = ops_lib::v0::write(format, &[full_model()]).unwrap();
        assert_eq!(
            ops_lib::v0::read(format, &out).unwrap(),
            vec![full_model()],
            "{} round-trip changed the model",
            format.name()
        );
    }
}

#[test]
fn writing_is_idempotent() {
    // A formatter (or a `--fix`) rewrites a file it just wrote; the bytes
    // must not churn. Byte layout is per implementation — OPS fixes the
    // model, not the bytes — so this checks stability, not a canonical form.
    for format in [Format::Json, Format::Yaml, Format::Jsonl] {
        let first = ops_lib::v0::write(format, &[full_model()]).unwrap();
        let parsed = ops_lib::v0::read(format, &first).unwrap();
        let again = ops_lib::v0::write(format, &parsed).unwrap();
        assert_eq!(
            first,
            again,
            "{} write -> read -> write is not byte-stable",
            format.name()
        );
    }
}

#[test]
fn invalid_models_are_rejected_before_writing() {
    let forest = vec![
        Task::new("Work".into(), Status::Open),
        Task::new("Personal".into(), Status::Open),
    ];
    let invalid = [
        (
            vec![
                Task {
                    id: Some("x".into()),
                    ..Task::new("A".into(), Status::Open)
                },
                Task {
                    id: Some("x".into()),
                    ..Task::new("B".into(), Status::Open)
                },
            ],
            "duplicate-id",
        ),
        (
            vec![Task {
                title: "R".into(),
                status: Status::Open,
                metadata: Some(meta(json!({ "bad-key": true }))),
                ..Default::default()
            }],
            "invalid-metadata-key",
        ),
        (
            vec![Task {
                title: "R".into(),
                status: Status::Open,
                subtasks: vec![Task {
                    version: Some("0".into()),
                    ..Task::new("C".into(), Status::Open)
                }],
                ..Default::default()
            }],
            "version-on-subtask",
        ),
        (
            vec![Task {
                version: Some("1".into()),
                ..Task::new("R".into(), Status::Open)
            }],
            "unknown-version",
        ),
    ];
    for (tasks, code) in invalid {
        let err = ops_lib::v0::write(Format::Json, &tasks).unwrap_err();
        assert_eq!(err.code(), code);
    }
    for format in [Format::Json, Format::Yaml] {
        let err = ops_lib::v0::write(format, &forest).unwrap_err();
        assert_eq!(err.code(), "single-root-expected");
    }
}

#[test]
fn the_full_model_covers_every_field() {
    let task = full_model();
    assert!(task.id.is_some() && task.version.is_some() && task.notes.is_some());
    assert!(task.metadata.is_some());
    assert_eq!(task.subtasks.len(), 2);
    assert_eq!(task.subtasks[0].subtasks.len(), 1);
}
