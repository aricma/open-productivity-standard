//! The model builders that match the standard's example documents.

#![allow(dead_code)]

use ops_lib::{Status, Task};
use serde_json::{Map, Value, json};

/// `json!` yields a Value; Task.metadata is a Map.
pub fn meta(v: Value) -> Map<String, Value> {
    v.as_object().unwrap().clone()
}

/// The model behind the standard's example documents (tree.json,
/// yaml.yaml, jsonl.jsonl, ...).
pub fn example_model() -> Task {
    Task {
        title: "Acme product backlog".into(),
        status: Status::Open,
        version: Some("0".into()),
        id: Some("export-root".into()),
        metadata: Some(meta(json!({
            "created_at": "2026-01-01T09:00:00Z",
            "updated_at": "2026-07-31T18:00:00Z",
            "url": "https://acme.example/backlog",
        }))),
        subtasks: vec![Task {
            id: Some("t1".into()),
            title: "Fix memory leak in auth service".into(),
            status: Status::Open,
            metadata: Some(meta(json!({
                "priority": "high",
                "start_date": "2026-07-01",
                "due_date": "2026-08-15",
                "tags": ["infrastructure", "auth"],
                "status": "In review",
                "assignee": "ada",
            }))),
            subtasks: vec![
                Task {
                    id: Some("t1r".into()),
                    title: "Reproduce the leak".into(),
                    status: Status::Open,
                    metadata: Some(meta(json!({
                        "priority": "high",
                        "location": "Remote",
                    }))),
                    ..Default::default()
                },
                Task {
                    id: Some("t1a".into()),
                    title: "Ship the fix".into(),
                    status: Status::Done,
                    metadata: Some(meta(json!({
                        "priority": "high",
                        "estimates": 3,
                        "attachments": ["notes/fix-auth.pdf"],
                        "completed_at": "2026-08-15T17:00:00Z",
                    }))),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The maximal Task model: every field populated, two nesting levels
/// (matches `local/valid/full-model.{json,yaml,jsonl}`).
pub fn full_model() -> Task {
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

/// Flat forms never attach an id to roots.
pub fn example_root_no_id() -> Task {
    let mut t = example_model();
    t.id = None;
    t
}

/// The `Work` root of the multi-root examples (t1's subtree is shared
/// with the single-root examples).
pub fn work_model() -> Task {
    let mut t = example_root_no_id();
    t.title = "Work".into();
    t
}

/// A leaf task with an id, as found in the flat examples.
pub fn chore(id: &str, title: &str, status: Status) -> Task {
    Task {
        id: Some(id.into()),
        title: title.into(),
        status,
        ..Default::default()
    }
}

/// The `Personal` root of the multi-root JSONL example.
pub fn personal_model() -> Task {
    Task {
        title: "Personal".into(),
        status: Status::Open,
        version: Some("0".into()),
        subtasks: vec![
            chore("p1", "Buy groceries", Status::Open),
            chore("p2", "Call dentist", Status::Done),
        ],
        ..Default::default()
    }
}

/// The empty export: a root with no subtasks.
pub fn empty_model() -> Task {
    Task {
        title: "Empty export".into(),
        status: Status::Open,
        version: Some("0".into()),
        ..Default::default()
    }
}
