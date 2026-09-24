//! Performance tripwire: read/write time across scales, per format.
//!
//! Not a benchmark suite. It prints a table and then asserts only that
//! time scales roughly linearly in the input size. Run with
//! `mask test --profile performance` (which adds `--nocapture`).

use ops_lib::v0::limits::{MAX_CHILDREN, MAX_TASKS};
use ops_lib::v0::{self, Format, Status, Task};
use serde_json::{Map, Value, json};
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
    let metadata: Map<String, Value> = json!({
        "priority": "high",
        "tags": ["infra", "auth"],
        "count": i,
        "active": true,
        "nested": { "key": "value" },
    })
    .as_object()
    .expect("object literal")
    .clone();
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
        for format in [Format::Json, Format::Yaml, Format::Jsonl] {
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
