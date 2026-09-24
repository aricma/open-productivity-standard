//! Boundary and security tests: the guardrails in `ops_lib::v0::limits`
//! and hostile input shapes. Every size derives from a constant, so
//! raising one moves the test with it.

use ops_lib::v0::limits::*;
use ops_lib::v0::{self, Format, Status, Task};

/// The document must be rejected with this code.
fn rejected(input: &str, format: Format, code: &str) {
    let err = v0::read(format, input).unwrap_err();
    assert_eq!(
        err.code(),
        code,
        "expected {code} for {} bytes",
        input.len()
    );
}

#[test]
fn document_size_guard() {
    let base = r#"{"title":"t","status":"open"}"#;
    let mut doc = String::with_capacity(MAX_INPUT_BYTES);
    doc.push_str(base);
    doc.push_str(&" ".repeat(MAX_INPUT_BYTES - base.len()));
    assert_eq!(doc.len(), MAX_INPUT_BYTES);
    assert!(v0::read(Format::Json, &doc).is_ok());

    doc.push(' ');
    rejected(&doc, Format::Json, "input-too-large");
}

fn jsonl_leaves(n: usize) -> String {
    (0..n)
        .map(|i| format!(r#"{{"id":"t{i}","title":"t","status":"open"}}"#))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn task_count_guard() {
    assert!(v0::read(Format::Jsonl, &jsonl_leaves(MAX_TASKS)).is_ok());
    rejected(
        &jsonl_leaves(MAX_TASKS + 1),
        Format::Jsonl,
        "too-many-tasks",
    );
}

fn tree_with_children(n: usize) -> String {
    let kids = std::iter::repeat_n(r#"{"title":"c","status":"open"}"#, n)
        .collect::<Vec<_>>()
        .join(",");
    format!(r#"{{"title":"root","status":"open","subtasks":[{kids}]}}"#)
}

#[test]
fn child_count_guard() {
    assert!(v0::read(Format::Json, &tree_with_children(MAX_CHILDREN)).is_ok());
    rejected(
        &tree_with_children(MAX_CHILDREN + 1),
        Format::Json,
        "too-many-children",
    );
}

/// A flat id-chain: shallow lines, deep tree. `records` tasks means a
/// built tree of depth `records - 1`.
fn id_chain(records: usize) -> String {
    (0..records)
        .map(|i| {
            if i + 1 < records {
                format!(
                    r#"{{"id":"r{i}","title":"t","status":"open","subtasks":["r{}"]}}"#,
                    i + 1
                )
            } else {
                format!(r#"{{"id":"r{i}","title":"t","status":"open"}}"#)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn depth_guard_on_flat_chain() {
    assert!(v0::read(Format::Jsonl, &id_chain(MAX_DEPTH + 1)).is_ok());
    rejected(&id_chain(MAX_DEPTH + 2), Format::Jsonl, "nesting-too-deep");
}

/// A chain built iteratively, since building must not recurse either.
fn deep(levels: usize) -> Task {
    let mut task = Task::new("t".into(), Status::Open);
    for _ in 1..levels {
        let mut parent = Task::new("t".into(), Status::Open);
        parent.subtasks.push(task);
        task = parent;
    }
    task
}

#[test]
fn depth_guard_on_in_memory_write() {
    assert!(v0::write(Format::Json, &[deep(MAX_DEPTH + 1)]).is_ok());
    let err = v0::write(Format::Json, &[deep(MAX_DEPTH + 2)]).unwrap_err();
    assert_eq!(err.code(), "nesting-too-deep");
}
