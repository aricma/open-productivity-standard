//! JSONL serialization (flat/streamable forest).

use super::helper::{forest, records};
use super::helper::{validate_forest, validated_tasks};
use super::raw::{RawTask, build_flat_task};
use crate::doc::OpsDoc;
use crate::error::Error;
use crate::model::task::Task;

fn parse(input: &str) -> Result<Vec<Task>, Error> {
    let mut records_vec = Vec::new();
    for (idx, line) in input.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let raw: RawTask = serde_json::from_str(line)
            .map_err(|e| Error::Json(format!("line {}: {e}", idx + 1)))?;
        records_vec.push(build_flat_task(raw)?);
    }
    forest(records_vec)
}

fn write(tasks: &[Task]) -> Result<String, Error> {
    let mut out = String::new();
    for r in records(tasks) {
        out.push_str(&serde_json::to_string(&r).map_err(|e| Error::Json(e.to_string()))?);
        out.push('\n');
    }
    Ok(out)
}

pub struct Jsonl;

impl OpsDoc for Jsonl {
    fn parse(self, input: &str) -> Result<Vec<Task>, Error> {
        validated_tasks(parse(input)?)
    }

    fn write(self, tasks: &[Task]) -> Result<String, Error> {
        validate_forest(tasks).map_err(Error::Validation)?;
        write(tasks)
    }
}
