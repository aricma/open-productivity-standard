//! Boundary and security tests: the guardrails in `ops_lib::v0::limits`
//! and hostile input shapes. Every size derives from a constant, so
//! raising one moves the test with it.

use ops_lib::v0::limits::*;
use ops_lib::v0::{self, Format, Metadata, MetadataValue, Status, Task};

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

/// `detect_version` parses too, so it takes the same input bound.
#[test]
fn detect_version_guard() {
    let base = r#"{"title":"t","status":"open"}"#;
    let mut doc = String::with_capacity(MAX_INPUT_BYTES);
    doc.push_str(base);
    doc.push_str(&" ".repeat(MAX_INPUT_BYTES - base.len()));
    assert!(v0::detect_version(Format::Json, &doc).is_ok());
    doc.push(' ');
    assert_eq!(
        v0::detect_version(Format::Json, &doc).unwrap_err().code(),
        "input-too-large"
    );
}

/// Detection is a probe, not a validation pass: the input bound covers the
/// line-by-line scan, and a document with many tasks is still detectable.
#[test]
fn detect_version_is_not_a_validation_pass() {
    let lines = std::iter::repeat_n(r#"{"id":"t","title":"t","status":"open"}"#, MAX_TASKS + 1)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(lines.len() < MAX_INPUT_BYTES, "{} bytes", lines.len());
    assert_eq!(
        v0::read(Format::Jsonl, &lines).unwrap_err().code(),
        "too-many-tasks"
    );
    assert!(v0::detect_version(Format::Jsonl, &lines).is_ok());
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

/// A nested chain `levels` deep, as text.
fn json_chain(levels: usize) -> String {
    let mut doc = String::new();
    for _ in 0..levels - 1 {
        doc.push_str(r#"{"title":"t","status":"open","subtasks":["#);
    }
    doc.push_str(r#"{"title":"t","status":"open"}"#);
    for _ in 0..levels - 1 {
        doc.push_str("]}");
    }
    doc
}

/// The same chains as a forest: with two or more roots the document gains an
/// array wrapper, which costs one level of the parser's budget.
fn json_forest(roots: usize, levels: usize) -> String {
    if roots == 1 {
        json_chain(levels)
    } else {
        format!("[{},{}]", json_chain(levels), json_chain(levels))
    }
}

/// A tree document is only as deep as a reader can parse it back, so the bound
/// holds from both sides: at it the document round-trips, whatever its shape,
/// and past it a writer refuses by the bound while a reader refuses because the
/// parser's budget is reached first.
#[test]
fn the_depth_bound_is_what_a_tree_document_can_carry() {
    for format in [Format::Json, Format::Yaml] {
        for roots in [1, 2] {
            let at = json_forest(roots, MAX_DEPTH + 1);
            let tasks = v0::read(format, &at).unwrap_or_else(|e| {
                panic!("{} with {roots} root(s): at MAX_DEPTH: {e}", format.name())
            });
            let out = v0::write(format, &tasks).unwrap_or_else(|e| {
                panic!(
                    "{} with {roots} root(s): writing it back: {e}",
                    format.name()
                )
            });
            assert_eq!(
                v0::read(format, &out).unwrap(),
                tasks,
                "{} with {roots} root(s): a document at MAX_DEPTH did not round-trip",
                format.name()
            );
        }

        // One level deeper: a reader refuses whatever the reason, and a writer
        // refuses because of the bound.
        assert!(
            v0::read(format, &json_forest(2, MAX_DEPTH + 2)).is_err(),
            "{}: past MAX_DEPTH was accepted",
            format.name()
        );
        let mut deeper = Task::new("root".into(), Status::Open);
        deeper.subtasks = v0::read(format, &json_chain(MAX_DEPTH + 1)).unwrap();
        assert_eq!(
            v0::write(format, &[deeper]).unwrap_err().code(),
            "nesting-too-deep",
            "{}",
            format.name()
        );
    }
}

/// The limits count bytes, not characters, so a multi-byte field is measured
/// by its UTF-8 length.
#[test]
fn field_limits_count_bytes_not_characters() {
    let at = MAX_TITLE_BYTES / '😀'.len_utf8();
    let ok = format!(r#"{{"title":"{}","status":"open"}}"#, "😀".repeat(at));
    assert!(
        v0::read(Format::Json, &ok).is_ok(),
        "{} bytes",
        "😀".repeat(at).len()
    );
    let over = format!(r#"{{"title":"{}","status":"open"}}"#, "😀".repeat(at + 1));
    rejected(&over, Format::Json, "field-too-long");
}

/// A metadata value's bound is its *serialized* size, so escaping counts: each
/// quote inside the value costs two bytes on the wire, plus the two around it.
#[test]
fn metadata_value_limit_counts_escaped_bytes() {
    let at = (MAX_METADATA_VALUE_BYTES - 2) / 2;
    let doc = metadata_doc(&format!(
        r#""blob":{}"#,
        serde_json::to_string(&"\"".repeat(at)).expect("string")
    ));
    assert!(v0::read(Format::Json, &doc).is_ok(), "{} bytes", doc.len());
    let doc = metadata_doc(&format!(
        r#""blob":{}"#,
        serde_json::to_string(&"\"".repeat(at + 1)).expect("string")
    ));
    rejected(&doc, Format::Json, "metadata-value-too-large");
}

/// A CSV document is `rows × columns` cells, and a key only one task carries
/// still needs a cell in every row. So a model the per-task limits allow can
/// ask for a document far past what this library would read back — the writer
/// refuses rather than building it.
#[test]
fn a_runaway_csv_write_is_refused() {
    let mut tasks = Vec::new();
    for i in 0..2_000 {
        let mut metadata = Metadata::new();
        for k in 0..8 {
            metadata.insert(format!("t{i:05}_k{k}"), MetadataValue::String("v".into()));
        }
        tasks.push(Task {
            title: format!("t{i}"),
            status: Status::Open,
            id: Some(format!("t{i}")),
            metadata: Some(metadata),
            ..Default::default()
        });
    }
    // The same model is fine in a format that does not spread keys over
    // columns, so it is the mapping, not the model, that is too big.
    assert!(v0::write(Format::Json, &tasks).is_ok());
    assert_eq!(
        v0::write(Format::Csv, &tasks).unwrap_err().code(),
        "output-too-large"
    );
}
