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

#[test]
fn title_guard() {
    let ok = format!(
        r#"{{"title":"{}","status":"open"}}"#,
        "x".repeat(MAX_TITLE_BYTES)
    );
    assert!(v0::read(Format::Json, &ok).is_ok());
    rejected(
        &format!(
            r#"{{"title":"{}","status":"open"}}"#,
            "x".repeat(MAX_TITLE_BYTES + 1)
        ),
        Format::Json,
        "field-too-long",
    );
}

#[test]
fn notes_guard() {
    let ok = format!(
        r#"{{"title":"t","status":"open","notes":"{}"}}"#,
        "x".repeat(MAX_NOTES_BYTES)
    );
    assert!(v0::read(Format::Json, &ok).is_ok());
    rejected(
        &format!(
            r#"{{"title":"t","status":"open","notes":"{}"}}"#,
            "x".repeat(MAX_NOTES_BYTES + 1)
        ),
        Format::Json,
        "field-too-long",
    );
}

#[test]
fn id_guard() {
    let ok = format!(
        r#"{{"id":"{}","title":"t","status":"open"}}"#,
        "x".repeat(MAX_ID_BYTES)
    );
    assert!(v0::read(Format::Json, &ok).is_ok());
    rejected(
        &format!(
            r#"{{"id":"{}","title":"t","status":"open"}}"#,
            "x".repeat(MAX_ID_BYTES + 1)
        ),
        Format::Json,
        "field-too-long",
    );
}

#[test]
fn version_guard() {
    let ok = r#"{"version":"0","title":"t","status":"open"}"#;
    assert!(v0::read(Format::Json, ok).is_ok());
    rejected(
        &format!(
            r#"{{"version":"{}","title":"t","status":"open"}}"#,
            "x".repeat(MAX_VERSION_BYTES + 1)
        ),
        Format::Json,
        "field-too-long",
    );
}

fn metadata_doc(pairs: &str) -> String {
    format!(r#"{{"title":"t","status":"open","metadata":{{{pairs}}}}}"#)
}

#[test]
fn metadata_pair_count_guard() {
    let pairs = |n: usize| {
        (0..n)
            .map(|i| format!(r#""k{i:03}":1"#))
            .collect::<Vec<_>>()
            .join(",")
    };
    assert!(v0::read(Format::Json, &metadata_doc(&pairs(MAX_METADATA_PAIRS))).is_ok());
    rejected(
        &metadata_doc(&pairs(MAX_METADATA_PAIRS + 1)),
        Format::Json,
        "too-many-metadata-pairs",
    );
}

#[test]
fn metadata_key_guard() {
    let entry = |len: usize| format!(r#""{}":1"#, "a".repeat(len));
    assert!(v0::read(Format::Json, &metadata_doc(&entry(MAX_METADATA_KEY_BYTES))).is_ok());
    rejected(
        &metadata_doc(&entry(MAX_METADATA_KEY_BYTES + 1)),
        Format::Json,
        "metadata-key-too-long",
    );
}

#[test]
fn metadata_value_guard() {
    // A JSON string's serialized size is its content plus two quotes.
    let value = |n: usize| format!(r#""{}""#, "x".repeat(n));
    let at = metadata_doc(&format!(
        r#""blob":{}"#,
        value(MAX_METADATA_VALUE_BYTES - 2)
    ));
    assert!(v0::read(Format::Json, &at).is_ok());
    let over = metadata_doc(&format!(
        r#""blob":{}"#,
        value(MAX_METADATA_VALUE_BYTES - 1)
    ));
    rejected(&over, Format::Json, "metadata-value-too-large");
}

#[test]
fn deeply_nested_tree_documents_are_rejected() {
    let mut doc = String::new();
    for _ in 0..1_000 {
        doc.push_str(r#"{"title":"t","status":"open","subtasks":["#);
    }
    doc.push_str(r#"{"title":"t","status":"open"}"#);
    for _ in 0..1_000 {
        doc.push_str("]}");
    }
    assert!(v0::read(Format::Json, &doc).is_err());
    assert!(v0::read(Format::Yaml, &doc).is_err());
}

#[test]
fn yaml_alias_bomb_is_rejected() {
    let mut doc = String::from("a: &a [\"x\"]\n");
    let mut prev = "a".to_string();
    for i in 0..10 {
        let name = format!("b{i}");
        let refs = std::iter::repeat_n(format!("*{prev}"), 4)
            .collect::<Vec<_>>()
            .join(",");
        doc.push_str(&format!("{name}: &{name} [{refs}]\n"));
        prev = name;
    }
    assert!(v0::read(Format::Yaml, &doc).is_err());
}

#[test]
fn malformed_json_is_rejected() {
    let valid = r#"{"title":"t","status":"open"}"#;
    let bad = [
        format!("\u{feff}{valid}"),
        format!("{valid} trailing"),
        format!("{valid}\u{0}"),
        "{".to_string(),
    ];
    for input in bad {
        assert!(v0::read(Format::Json, &input).is_err(), "{input:?}");
    }
}
