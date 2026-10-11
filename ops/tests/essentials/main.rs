//! Model-level tests: cases that cannot be expressed as a document in the
//! portable corpus — building an invalid in-memory model, the full-model
//! round-trip, and writer idempotence.

use ops_lib::v0::{Format, Metadata, MetadataValue, Number, Status, Task};
use serde_json::{Value, json};
use std::collections::BTreeSet;

fn meta(value: Value) -> Metadata {
    serde_json::from_value(value).unwrap()
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
                    status: Status::Done,
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
    for &format in Format::ALL {
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
    for &format in Format::ALL {
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
            "unsupported-version",
        ),
        (vec![Task::new(String::new(), Status::Open)], "empty-title"),
        (
            vec![Task::new("two\nlines".into(), Status::Open)],
            "title-has-line-break",
        ),
    ];
    for (tasks, code) in invalid {
        let err = ops_lib::v0::write(Format::Json, &tasks).unwrap_err();
        assert_eq!(err.code(), code);
    }
}

/// A value that carries nothing — an empty string, list, map, or `null` — is
/// absent, in `metadata` and in the fields that can hold an empty string. So a
/// writer leaves it out, and a model that came back from the reader writes the
/// same bytes again.
#[test]
fn a_value_that_carries_nothing_is_not_written() {
    let mut metadata = Metadata::new();
    metadata.insert("priority".into(), MetadataValue::String("high".into()));
    metadata.insert("blank".into(), MetadataValue::String(String::new()));
    metadata.insert("nothing".into(), MetadataValue::Null);
    metadata.insert("none".into(), MetadataValue::List(Vec::new()));
    metadata.insert("void".into(), MetadataValue::Map(Metadata::new()));
    let written = vec![Task {
        title: "Root".into(),
        status: Status::Open,
        id: Some(String::new()),
        version: Some(String::new()),
        notes: Some(String::new()),
        metadata: Some(metadata),
        ..Default::default()
    }];
    let expected = vec![Task {
        title: "Root".into(),
        status: Status::Open,
        metadata: Some(meta(json!({ "priority": "high" }))),
        ..Default::default()
    }];

    for &format in Format::ALL {
        let first = ops_lib::v0::write(format, &written)
            .unwrap_or_else(|e| panic!("{}: {e}", format.name()));
        let back =
            ops_lib::v0::read(format, &first).unwrap_or_else(|e| panic!("{}: {e}", format.name()));
        assert_eq!(
            back,
            expected,
            "{} carried a value that says nothing into the model",
            format.name()
        );
        assert_eq!(
            ops_lib::v0::write(format, &back).unwrap(),
            first,
            "{}: write -> read -> write is not byte-stable",
            format.name()
        );
    }
}

/// A flat format carries the hierarchy in id lists, and an empty id is no id at
/// all, so a subtask with one cannot be referenced: the writer refuses rather
/// than emitting a document that reads back differently.
#[test]
fn a_flat_write_refuses_a_subtask_with_an_empty_id() {
    let mut root = Task::new("Root".into(), Status::Open);
    root.id = Some("r".into());
    let mut child = Task::new("Child".into(), Status::Open);
    child.id = Some(String::new());
    root.subtasks = vec![child];

    for format in [Format::Jsonl, Format::Csv] {
        assert_eq!(
            ops_lib::v0::write(format, &[root.clone()])
                .unwrap_err()
                .code(),
            "subtask-without-id",
            "{}",
            format.name()
        );
    }

    // A tree format carries the hierarchy itself, and reads the empty id as no
    // id: the child comes back without one.
    for format in [Format::Json, Format::Yaml] {
        let out = ops_lib::v0::write(format, &[root.clone()]).unwrap();
        let back = ops_lib::v0::read(format, &out).unwrap();
        assert_eq!(back[0].subtasks[0].id, None, "{}", format.name());
    }
}

#[test]
fn a_forest_roundtrips_through_json_and_yaml() {
    let forest = vec![
        Task::new("Work".into(), Status::Open),
        Task::new("Personal".into(), Status::Done),
    ];
    for format in [Format::Json, Format::Yaml] {
        let out = ops_lib::v0::write(format, &forest).unwrap();
        assert_eq!(
            ops_lib::v0::read(format, &out).unwrap(),
            forest,
            "{} forest round-trip changed the model",
            format.name()
        );
    }
    // One root stays an object, so `[a]` normalizes to an object.
    let one = ops_lib::v0::read(Format::Json, "[{\"title\":\"a\",\"status\":\"open\"}]").unwrap();
    let out = ops_lib::v0::write(Format::Json, &one).unwrap();
    assert!(out.trim_start().starts_with('{'), "{out}");
    assert_eq!(ops_lib::v0::read(Format::Json, &out).unwrap(), one);
}

#[test]
fn an_empty_document_is_an_empty_forest() {
    assert_eq!(ops_lib::v0::read(Format::Json, "[]").unwrap(), vec![]);
    let out = ops_lib::v0::write(Format::Json, &[]).unwrap();
    assert_eq!(out.trim(), "[]");
}

#[test]
fn the_full_model_covers_every_field() {
    let task = full_model();
    assert!(task.id.is_some() && task.version.is_some() && task.notes.is_some());
    assert!(task.metadata.is_some());
    assert_eq!(task.subtasks.len(), 2);
    assert_eq!(task.subtasks[0].subtasks.len(), 1);
}

#[test]
fn a_done_task_with_an_open_subtask_is_rejected() {
    let tasks = vec![Task {
        title: "parent".into(),
        status: Status::Done,
        subtasks: vec![Task::new("child".into(), Status::Open)],
        ..Default::default()
    }];
    assert_eq!(
        ops_lib::v0::write(Format::Json, &tasks).unwrap_err().code(),
        "open-subtask"
    );
}

/// A flat format references tasks by id, so a writer must refuse a model whose
/// subtasks have none rather than emit a document that reads back differently.
///
/// Ids are for *reaching* a task, not for being one: a task without an id is
/// fine as long as nothing has to point at it. The two ways a reference can go
/// wrong are different errors — a child with no id cannot be written, an id
/// that reaches nothing cannot be read.
#[test]
fn a_flat_write_needs_an_id_only_on_subtasks() {
    // Roots without ids are valid in every format, flat ones included.
    let roots = vec![
        Task::new("One".into(), Status::Open),
        Task::new("Two".into(), Status::Done),
    ];
    for format in [Format::Json, Format::Yaml, Format::Jsonl, Format::Csv] {
        let out = ops_lib::v0::write(format, &roots)
            .unwrap_or_else(|e| panic!("{}: roots need no ids: {e}", format.name()));
        assert_eq!(
            ops_lib::v0::read(format, &out).unwrap(),
            roots,
            "{}",
            format.name()
        );
    }

    // A nested format carries the hierarchy itself, so children need none.
    let mut root = Task::new("Root".into(), Status::Open);
    root.subtasks = vec![Task::new("Child".into(), Status::Open)];
    let nested = vec![root];
    for format in [Format::Json, Format::Yaml] {
        assert!(
            ops_lib::v0::write(format, &nested).is_ok(),
            "{}",
            format.name()
        );
    }

    // A flat one has only ids to carry the hierarchy, so a child without one
    // cannot be referenced: the writer refuses rather than dropping the child.
    for format in [Format::Jsonl, Format::Csv] {
        assert_eq!(
            ops_lib::v0::write(format, &nested).unwrap_err().code(),
            "subtask-without-id",
            "{}",
            format.name()
        );
    }

    // With the id it writes, and the tree comes back.
    let mut root = Task::new("Root".into(), Status::Open);
    let mut child = Task::new("Child".into(), Status::Open);
    child.id = Some("c".into());
    root.subtasks = vec![child];
    let with_ids = vec![root];
    for format in [Format::Jsonl, Format::Csv] {
        let out = ops_lib::v0::write(format, &with_ids).unwrap();
        assert_eq!(
            ops_lib::v0::read(format, &out).unwrap(),
            with_ids,
            "{}",
            format.name()
        );
    }

    // An id that reaches nothing is the *reader's* error, and a different one.
    let dangling = r#"{"id":"a","title":"A","status":"open","subtasks":["gone"]}"#;
    assert_eq!(
        ops_lib::v0::read(Format::Jsonl, dangling)
            .unwrap_err()
            .code(),
        "missing-reference"
    );
}

/// Which spec versions a tool accepts is its own policy, not the standard's:
/// the standard only asks that a declaration be findable. This library
/// implements `0`, so a document claiming another is refused rather than read
/// with the wrong model.
#[test]
fn a_version_this_library_does_not_implement_is_refused() {
    let json = r#"{"version":"9","title":"t","status":"open"}"#;
    // Detection is a probe: it reports what the document claims, whatever it is.
    assert_eq!(
        ops_lib::v0::detect_version(Format::Json, json).unwrap(),
        Some("9".to_string())
    );
    assert_eq!(
        ops_lib::v0::read(Format::Json, json).unwrap_err().code(),
        "unsupported-version"
    );
    let jsonl = r#"{"version":"9","title":"t","status":"open"}"#;
    assert_eq!(
        ops_lib::v0::read(Format::Jsonl, jsonl).unwrap_err().code(),
        "unsupported-version"
    );
    // The version it does implement is read without complaint.
    assert!(
        ops_lib::v0::read(
            Format::Json,
            r#"{"version":"0","title":"t","status":"open"}"#
        )
        .is_ok()
    );
}

/// A tiny deterministic PRNG (xorshift64*), so a failing case reproduces from
/// its seed alone and the suite stays dependency-free.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn maybe(&mut self) -> bool {
        self.below(2) == 0
    }
}

/// A title is one non-empty line, so no line breaks here.
const TITLE_CHARS: &[char] = &['a', 'Z', '3', ' ', ',', '"', '#', '-', ':', ')'];

/// Notes and string values may say anything: a line break, a comma, a quote,
/// a leading space, something that parses as JSON.
const TEXT_CHARS: &[char] = &['a', 'Z', '3', ' ', ',', '"', '#', '-', ':', '\n', '(', ')'];

/// Metadata keys must match the charset, and a key may collide with a reserved
/// column name or start with the `metadata_` prefix.
const KEYS: &[&str] = &[
    "due_date",
    "priority",
    "tags",
    "assignee",
    "metadata_thing",
    "title",
    "jira_13",
];

fn text(rng: &mut Rng, chars: &[char], max: usize) -> String {
    let len = 1 + rng.below(max);
    (0..len).map(|_| chars[rng.below(chars.len())]).collect()
}

fn metadata_value(rng: &mut Rng) -> MetadataValue {
    match rng.below(6) {
        // Something that parses as JSON: the CSV writer has to recognise it and
        // route it to the `metadata` column instead of a bare cell.
        0 => {
            MetadataValue::String(["3", "true", "[]", "\"q\"", "-1.5e3"][rng.below(5)].to_string())
        }
        1 => MetadataValue::String(text(rng, TEXT_CHARS, 6)),
        2 => MetadataValue::Number(Number::from(rng.below(1_000) as i64)),
        3 => MetadataValue::Bool(rng.maybe()),
        4 => MetadataValue::List(vec![MetadataValue::String(text(rng, TEXT_CHARS, 4))]),
        _ => {
            let mut inner = Metadata::new();
            inner.insert(
                "k".to_string(),
                MetadataValue::String(text(rng, TEXT_CHARS, 4)),
            );
            MetadataValue::Map(inner)
        }
    }
}

fn metadata(rng: &mut Rng) -> Option<Metadata> {
    let pairs = rng.below(4);
    if pairs == 0 {
        return None;
    }
    // Without replacement, so the loop is bounded.
    let mut keys = KEYS.to_vec();
    for i in (1..keys.len()).rev() {
        keys.swap(i, rng.below(i + 1));
    }
    let mut out = Metadata::new();
    for key in keys.into_iter().take(pairs) {
        out.insert(key.to_string(), metadata_value(rng));
    }
    Some(out)
}

/// One valid task: children first, so the done rule can be honoured, and an id
/// on every subtask, so a flat document can reference it.
fn task(rng: &mut Rng, depth: usize, next_id: &mut usize) -> Task {
    let mut subtasks = Vec::new();
    if depth < 3 {
        for _ in 0..rng.below(3) {
            subtasks.push(task(rng, depth + 1, next_id));
        }
    }
    let status = if subtasks.iter().any(|t| t.status == Status::Open) {
        Status::Open
    } else if rng.maybe() {
        Status::Done
    } else {
        Status::Open
    };
    let id = format!("t{}", *next_id);
    *next_id += 1;
    Task {
        title: text(rng, TITLE_CHARS, 8),
        status,
        id: Some(id),
        notes: if rng.maybe() {
            Some(text(rng, TEXT_CHARS, 12))
        } else {
            None
        },
        metadata: metadata(rng),
        subtasks,
        ..Default::default()
    }
}

fn forest(rng: &mut Rng) -> Vec<Task> {
    let mut next_id = 0;
    let mut roots: Vec<Task> = (0..rng.below(3))
        .map(|_| task(rng, 0, &mut next_id))
        .collect();
    for root in &mut roots {
        // A root never needs an id in any format.
        if rng.maybe() {
            root.id = None;
        }
        root.version = Some("0".to_string());
    }
    roots
}

/// The corpus pins the cases we thought of. This walks the space around them:
/// every value shape, every awkward character, the reserved-name and
/// `metadata_`-prefixed keys, roots with and without ids.
#[test]
fn generated_models_round_trip_through_every_format() {
    let mut seen: BTreeSet<&'static str> = BTreeSet::new();
    for seed in 1..=400u64 {
        let mut rng = Rng(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1);
        let model = forest(&mut rng);
        features(&model, &mut seen);
        for format in [Format::Json, Format::Yaml, Format::Jsonl, Format::Csv] {
            let doc = ops_lib::v0::write(format, &model)
                .unwrap_or_else(|e| panic!("seed {seed}: write {}: {e}", format.name()));
            let back = ops_lib::v0::read(format, &doc)
                .unwrap_or_else(|e| panic!("seed {seed}: read {}: {e}\n{doc}", format.name()));
            // A value that carries nothing is not a fixed point: the reader
            // reads it as absent and the writer leaves it out, so this
            // generator makes none (see
            // `a_value_that_carries_nothing_is_not_written`).
            assert_eq!(
                back,
                model,
                "seed {seed}: {} round-trip changed the model\n{doc}",
                format.name()
            );
        }
    }
    // A generated suite is only worth its runtime if it walks the space, so
    // say which corners it must have reached.
    let required = [
        "an empty forest",
        "a root without an id",
        "a done task",
        "a task with children",
        "a multi-line note",
        "a JSON-looking string value",
        "a list value",
        "a map value",
        "a reserved-name metadata key",
        "a metadata_-prefixed key",
    ];
    for what in required {
        assert!(seen.contains(what), "no generated model had {what}");
    }
}

/// Records which corners of the space a model reached.
fn features(model: &[Task], seen: &mut BTreeSet<&'static str>) {
    if model.is_empty() {
        seen.insert("an empty forest");
    }
    for task in model {
        if task.id.is_none() {
            seen.insert("a root without an id");
        }
        if task.status == Status::Done {
            seen.insert("a done task");
        }
        if !task.subtasks.is_empty() {
            seen.insert("a task with children");
        }
        if task.notes.as_deref().is_some_and(|n| n.contains('\n')) {
            seen.insert("a multi-line note");
        }
        if let Some(pairs) = &task.metadata {
            for (key, value) in pairs.iter() {
                if [
                    "title", "status", "id", "notes", "subtasks", "metadata", "version",
                ]
                .contains(&key.as_str())
                {
                    seen.insert("a reserved-name metadata key");
                }
                if key.starts_with("metadata_") {
                    seen.insert("a metadata_-prefixed key");
                }
                match value {
                    MetadataValue::String(s) if serde_json::from_str::<Value>(s).is_ok() => {
                        seen.insert("a JSON-looking string value");
                    }
                    MetadataValue::List(_) => {
                        seen.insert("a list value");
                    }
                    MetadataValue::Map(_) => {
                        seen.insert("a map value");
                    }
                    _ => {}
                }
            }
        }
        features(&task.subtasks, seen);
    }
}
