//! Performance tripwire: read/write time across scales, per format.
//!
//! Not a benchmark suite. It prints tables and asserts only that time is
//! proportional to the size a document or a limit bounds. Run with
//! `mask test --profile performance` (which adds `--nocapture`).

use ops_lib::v0::limits::*;
use ops_lib::v0::{self, Format, Metadata, MetadataValue, Status, Task};
use serde_json::json;
use std::time::{Duration, Instant};

/// Task counts to measure; anything the guardrails reject is skipped.
const SCALES: [usize; 5] = [0, 10, 100, 1_000, 10_000];
/// Runs per measurement; the fastest is kept, to drop scheduler noise.
const RUNS: usize = 3;
/// A 10x size step may take at most `10 * MARGIN`x.
const MARGIN: u32 = 2;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Op {
    Read,
    Write,
}

impl Op {
    fn label(self) -> &'static str {
        match self {
            Op::Read => "read",
            Op::Write => "write",
        }
    }
}

fn minimal_one(i: usize) -> Task {
    Task {
        id: Some(format!("t{i}")),
        ..Task::new("t".into(), Status::Open)
    }
}

fn dense_one(i: usize) -> Task {
    let metadata: Metadata = serde_json::from_value(json!({
        "priority": "high",
        "tags": ["infra", "auth"],
        "count": i,
        "active": true,
        "nested": { "key": "value" },
    }))
    .expect("object literal");
    Task {
        title: format!("Task {i}"),
        status: if i.is_multiple_of(2) {
            Status::Open
        } else {
            Status::Done
        },
        id: Some(format!("t{i}")),
        notes: Some("Some notes about this task.".into()),
        metadata: Some(metadata),
        ..Default::default()
    }
}

/// One root with `n` children, so tree formats have a single root and
/// flat formats a forest.
fn tasks(shape: &str, n: usize) -> Vec<Task> {
    let make = if shape == "dense" {
        dense_one
    } else {
        minimal_one
    };
    let mut root = make(0);
    root.subtasks = (1..=n).map(make).collect();
    vec![root]
}

fn best(op: Op, format: Format, tasks: &[Task], doc: &str) -> Duration {
    (0..RUNS)
        .map(|_| {
            let start = Instant::now();
            match op {
                Op::Write => {
                    v0::write(format, tasks).expect("write");
                }
                Op::Read => {
                    v0::read(format, doc).expect("read");
                }
            }
            start.elapsed()
        })
        .min()
        .expect("at least one run")
}

#[test]
fn read_and_write_scale_linearly() {
    let mut rows: Vec<(Format, Op, &'static str, usize, Duration)> = Vec::new();
    for shape in ["minimal", "dense"] {
        for format in [Format::Json, Format::Yaml, Format::Jsonl, Format::Csv] {
            for &n in &SCALES {
                if n > MAX_CHILDREN || n + 1 > MAX_TASKS {
                    continue;
                }
                let set = tasks(shape, n);
                let doc = v0::write(format, &set).expect("write");
                rows.push((
                    format,
                    Op::Write,
                    shape,
                    n,
                    best(Op::Write, format, &set, &doc),
                ));
                rows.push((
                    format,
                    Op::Read,
                    shape,
                    n,
                    best(Op::Read, format, &set, &doc),
                ));
            }
        }
    }

    println!(
        "\n{:<6} {:<6} {:<8} {:>7} {:>10} {:>12}",
        "format", "op", "shape", "n", "ms", "us/item"
    );
    for &(format, op, shape, n, d) in &rows {
        let per_item = if n == 0 {
            0.0
        } else {
            d.as_secs_f64() * 1e6 / n as f64
        };
        println!(
            "{:<6} {:<6} {:<8} {:>7} {:>10.3} {:>12.3}",
            format.name(),
            op.label(),
            shape,
            n,
            d.as_secs_f64() * 1e3,
            per_item
        );
    }

    for &(format, op, shape, n_small, small) in &rows {
        if !(100..MAX_TASKS).contains(&n_small) {
            continue;
        }
        let n_large = n_small * 10;
        let Some(&(_, _, _, _, large)) = rows
            .iter()
            .find(|r| r.0 == format && r.1 == op && r.2 == shape && r.3 == n_large)
        else {
            continue;
        };
        let allowed = small.as_nanos() as f64 * 10.0 * f64::from(MARGIN);
        assert!(
            large.as_nanos() as f64 <= allowed,
            "{} {} {}: n={n_large} took {large:?}, more than {}x the {small:?} at n={n_small}",
            format.name(),
            op.label(),
            shape,
            10 * MARGIN
        );
    }
}

/// Every limit promises that a datum at its bound costs time proportional to
/// the size it bounds, so each is measured at the bound and at half of it.
///
/// The allowance follows the *document*, not the scale: a nested document
/// grows faster than the datum does, because indentation deepens with depth,
/// and a bigger document is allowed to cost more — just not superlinearly.
fn assert_proportional(limit: &str, format: Format, at: impl Fn(usize) -> Vec<Task>, scale: usize) {
    let half = at(scale / 2);
    let full = at(scale);
    let half_doc = v0::write(format, &half).expect("write");
    let full_doc = v0::write(format, &full).expect("write");
    let growth = full_doc.len().max(1) as f64 / half_doc.len().max(1) as f64;
    let allowed = growth * f64::from(MARGIN);
    let read = |doc: &str| best(Op::Read, format, &[], doc).as_secs_f64() * 1e3;
    let write = |tasks: &[Task], doc: &str| best(Op::Write, format, tasks, doc).as_secs_f64() * 1e3;
    for (op, half_ms, full_ms) in [
        ("read", read(&half_doc), read(&full_doc)),
        ("write", write(&half, &half_doc), write(&full, &full_doc)),
    ] {
        println!(
            "{limit:<22} {op:<6} {scale:>8}  {half_ms:>9.2} -> {full_ms:>9.2} ms   ({:.2}x, allowed {allowed:.2}x for {growth:.2}x the document)",
            full_ms / half_ms.max(0.05),
        );
        assert!(
            full_ms <= half_ms * allowed + 5.0,
            "{limit}: {op} at {scale} took {full_ms:.2} ms, more than {allowed:.2}x the {half_ms:.2} ms at {}",
            scale / 2,
        );
    }
}

fn best_read(format: Format, doc: &str) -> f64 {
    (0..RUNS)
        .map(|_| {
            let start = Instant::now();
            v0::read(format, doc).expect("read");
            start.elapsed()
        })
        .min()
        .expect("at least one run")
        .as_secs_f64()
        * 1e3
}

/// A document of exactly `bytes`, most of it insignificant whitespace: the
/// input bound.
fn padded(bytes: usize) -> String {
    let base = r#"{"title":"t","status":"open"}"#;
    let mut doc = String::with_capacity(bytes);
    doc.push_str(base);
    doc.push_str(&" ".repeat(bytes - base.len()));
    doc
}

/// A chain `levels` deep: the depth bound.
fn chain(levels: usize) -> Vec<Task> {
    let mut task = Task::new("leaf".into(), Status::Open);
    for i in 1..levels {
        let mut parent = Task::new(format!("p{i}"), Status::Open);
        parent.subtasks = vec![task];
        task = parent;
    }
    vec![task]
}

/// One root carrying `children` children: the child bound.
fn bush(children: usize) -> Vec<Task> {
    let mut root = Task::new("root".into(), Status::Open);
    root.subtasks = (0..children)
        .map(|i| Task {
            id: Some(format!("c{i}")),
            ..Task::new(format!("c{i}"), Status::Open)
        })
        .collect();
    vec![root]
}

/// About `n` tasks in two levels — 100 roots of their share of the rest, so
/// the shape stays inside the depth and child bounds: the task bound.
fn many(n: usize) -> Vec<Task> {
    let roots = 100;
    let each = (n - roots) / roots;
    (0..roots)
        .map(|r| {
            let mut root = Task {
                id: Some(format!("r{r}")),
                ..Task::new(format!("r{r}"), Status::Open)
            };
            root.subtasks = (0..each)
                .map(|c| Task {
                    id: Some(format!("r{r}c{c}")),
                    ..Task::new(format!("c{c}"), Status::Open)
                })
                .collect();
            root
        })
        .collect()
}

/// `trees` chains, each `MAX_DEPTH + 1` levels deep: the depth bound,
/// multiplied. One chain at the bound is too small to time.
fn deep(trees: usize) -> Vec<Task> {
    (0..trees)
        .map(|_| chain(MAX_DEPTH + 1).into_iter().next().expect("one root"))
        .collect()
}

/// `tasks` tasks, each carrying the full set of metadata pairs — shared keys,
/// so the columns stay few: the pair bound, multiplied.
fn paired(tasks: usize) -> Vec<Task> {
    (0..tasks)
        .map(|i| {
            let mut metadata = Metadata::new();
            for k in 0..MAX_METADATA_PAIRS {
                metadata.insert(format!("k{k:02}"), MetadataValue::String("v".into()));
            }
            Task {
                title: format!("t{i}"),
                status: Status::Open,
                id: Some(format!("t{i}")),
                metadata: Some(metadata),
                ..Default::default()
            }
        })
        .collect()
}

/// `tasks` tasks, each carrying one value of exactly the bound — a JSON
/// string's serialized size is its content plus two quotes: the value bound.
fn valued(tasks: usize) -> Vec<Task> {
    let blob = "x".repeat(MAX_METADATA_VALUE_BYTES - 2);
    (0..tasks)
        .map(|i| {
            let mut metadata = Metadata::new();
            metadata.insert("blob".into(), MetadataValue::String(blob.clone()));
            Task {
                title: format!("t{i}"),
                status: Status::Open,
                id: Some(format!("t{i}")),
                metadata: Some(metadata),
                ..Default::default()
            }
        })
        .collect()
}

/// Every bounded field at its bound: title, notes, id, metadata key.
///
/// `version` has no entry here: the only version this library implements is
/// `"0"`, one byte, so its bound is unreachable until a longer version ships.
fn sized(tasks: usize) -> Vec<Task> {
    let title = "t".repeat(MAX_TITLE_BYTES);
    let notes = "n".repeat(MAX_NOTES_BYTES);
    let key = "k".repeat(MAX_METADATA_KEY_BYTES);
    let id_prefix = "i".repeat(MAX_ID_BYTES - 5);
    (0..tasks)
        .map(|i| {
            let mut metadata = Metadata::new();
            metadata.insert(key.clone(), MetadataValue::String("v".into()));
            Task {
                title: title.clone(),
                status: Status::Open,
                notes: Some(notes.clone()),
                id: Some(format!("{id_prefix}{i:05}")),
                metadata: Some(metadata),
                ..Default::default()
            }
        })
        .collect()
}

#[test]
fn input_bound_is_linear_in_bytes() {
    let half = best_read(Format::Json, &padded(MAX_INPUT_BYTES / 2));
    let full = best_read(Format::Json, &padded(MAX_INPUT_BYTES));
    println!(
        "{:<22} {:<6} {:>8}  {half:>9.2} -> {full:>9.2} ms   ({:.2}x, allowed {}x)",
        "input bytes",
        "read",
        MAX_INPUT_BYTES,
        full / half.max(0.05),
        2 * MARGIN
    );
    assert!(
        full <= half * f64::from(2 * MARGIN) + 5.0,
        "input bytes: read at {MAX_INPUT_BYTES} took {full:.2} ms, more than {}x the {half:.2} ms at half",
        2 * MARGIN
    );
}

#[test]
fn task_bound_is_proportional() {
    assert_proportional("tasks", Format::Jsonl, many, MAX_TASKS);
}

#[test]
fn depth_bound_is_proportional() {
    // A chain of `levels` nodes reaches depth `levels - 1`.
    assert_proportional("depth", Format::Json, deep, 200);
}

#[test]
fn child_bound_is_proportional() {
    assert_proportional("children", Format::Json, bush, MAX_CHILDREN);
}

#[test]
fn metadata_pair_bound_is_proportional() {
    // The bound is per task, so it is measured across many of them.
    assert_proportional("metadata pairs", Format::Csv, paired, 2_000);
}

#[test]
fn metadata_value_bound_is_proportional() {
    assert_proportional("metadata value", Format::Json, valued, 100);
}

#[test]
fn field_size_bounds_are_proportional() {
    assert_proportional("field sizes", Format::Json, sized, 100);
}
