//! Shared helpers for the serialization implementations: the semantic
//! validation the facade runs, the tree-format write constraint, and the
//! flat-format machinery (records are one task per line/row, hierarchy is
//! expressed by child-id lists, and the tree is rebuilt by resolving them
//! — shared children, missing references, and cycles are rejected here,
//! since a flat document with a broken graph cannot even be represented
//! as a tree).

use crate::v0::error::{Error, ValidationError};
use crate::v0::limits::{Limit, check, metadata_value_bytes};
use crate::v0::model::flat_task::FlatTask;
use crate::v0::model::status::Status;
use crate::v0::model::task::Task;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

#[allow(clippy::expect_used)]
static METADATA_KEY_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9_]{3,}$").expect("static pattern is valid"));

pub(crate) fn valid_metadata_key(key: &str) -> bool {
    METADATA_KEY_PATTERN.is_match(key)
}

fn walk(
    task: &Task,
    depth: usize,
    ids: &mut HashSet<String>,
    count: &mut usize,
) -> Result<(), ValidationError> {
    check(Limit::Depth, depth)?;
    *count += 1;
    check(Limit::Tasks, *count)?;
    check(Limit::Children, task.subtasks.len())?;
    check(Limit::Title, task.title.len())?;
    if task.title.is_empty() {
        return Err(ValidationError::EmptyTitle);
    }
    if task.title.contains(['\n', '\r']) {
        return Err(ValidationError::TitleHasLineBreak);
    }
    if let Some(notes) = &task.notes {
        check(Limit::Notes, notes.len())?;
    }
    if let Some(id) = &task.id {
        check(Limit::Id, id.len())?;
    }
    if let Some(version) = &task.version {
        check(Limit::Version, version.len())?;
    }
    if let Some(metadata) = &task.metadata {
        check(Limit::MetadataPairs, metadata.len())?;
        for (key, value) in metadata.iter() {
            if !valid_metadata_key(key) {
                return Err(ValidationError::InvalidMetadataKey(key.clone()));
            }
            check(Limit::MetadataKey, key.len())?;
            check(Limit::MetadataValue, metadata_value_bytes(value))?;
        }
    }
    if let Some(id) = task.id.as_deref().filter(|id| !id.is_empty())
        && !ids.insert(id.to_owned())
    {
        return Err(ValidationError::DuplicateId(id.to_owned()));
    }
    if task.status == Status::Done
        && let Some(child) = task.subtasks.iter().find(|c| c.status != Status::Done)
    {
        return Err(ValidationError::OpenSubtask {
            task: task.title.clone(),
            subtask: child.title.clone(),
        });
    }
    if depth > 0 && task.version.as_deref().is_some_and(|v| !v.is_empty()) {
        return Err(ValidationError::VersionOnSubtask(task.title.clone()));
    }
    for child in &task.subtasks {
        walk(child, depth + 1, ids, count)?;
    }
    Ok(())
}

/// Validate an in-memory forest against the semantic rules and the
/// structural guardrails.
pub(crate) fn validate_forest(tasks: &[Task]) -> Result<(), ValidationError> {
    let mut ids = HashSet::new();
    let mut count = 0;
    for t in tasks {
        walk(t, 0, &mut ids, &mut count)?;
    }
    Ok(())
}

/// A flat format references tasks by id, so every subtask needs one. A root
/// does not, and a nested format needs none at all. An empty id is no id, so it
/// is not a reference: the writer refuses rather than let the child out as a
/// root.
#[cfg_attr(not(any(feature = "jsonl", feature = "csv")), allow(dead_code))]
pub(crate) fn check_subtask_ids(tasks: &[Task]) -> Result<(), ValidationError> {
    fn walk(task: &Task) -> Result<(), ValidationError> {
        for child in &task.subtasks {
            if child.id.as_deref().is_none_or(str::is_empty) {
                return Err(ValidationError::SubtaskWithoutId {
                    task: task.title.clone(),
                    subtask: child.title.clone(),
                });
            }
            walk(child)?;
        }
        Ok(())
    }
    for task in tasks {
        walk(task)?;
    }
    Ok(())
}

/// Flatten a forest into records (pre-order, root first), shared by the
/// JSONL and CSV writers.
#[cfg_attr(not(any(feature = "jsonl", feature = "csv")), allow(dead_code))]
pub(crate) fn records(tasks: &[Task]) -> Vec<FlatTask> {
    let mut out = Vec::new();
    for t in tasks {
        flatten(t, &mut out);
    }
    out
}

#[cfg_attr(not(any(feature = "jsonl", feature = "csv")), allow(dead_code))]
fn flatten(task: &Task, out: &mut Vec<FlatTask>) {
    out.push(FlatTask::from_task(task));
    for child in &task.subtasks {
        flatten(child, out);
    }
}

/// Rebuild a forest from flat records, enforcing the graph rules:
/// unique ids, every referenced id exists, one parent per task, no
/// cycles, `version` on roots only.
#[cfg_attr(not(any(feature = "jsonl", feature = "csv")), allow(dead_code))]
pub(crate) fn forest(records: Vec<FlatTask>) -> Result<Vec<Task>, Error> {
    check(Limit::Tasks, records.len()).map_err(Error::Validation)?;
    let n = records.len();
    let mut by_id: HashMap<String, usize> = HashMap::new();
    for (i, r) in records.iter().enumerate() {
        // An empty id is no id, so it names nothing and collides with nothing.
        if let Some(id) = r.id.as_deref().filter(|id| !id.is_empty())
            && by_id.insert(id.to_owned(), i).is_some()
        {
            return Err(Error::Validation(ValidationError::DuplicateId(
                id.to_owned(),
            )));
        }
    }

    let mut children: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut referenced: Vec<bool> = vec![false; n];
    let mut parent: Vec<usize> = vec![usize::MAX; n];

    for (i, r) in records.iter().enumerate() {
        for cid in r.child_ids() {
            let Some(&j) = by_id.get(cid) else {
                return Err(Error::Validation(ValidationError::MissingReference(
                    cid.clone(),
                )));
            };
            if parent[j] != usize::MAX {
                return Err(Error::Validation(ValidationError::SharedChild(cid.clone())));
            }
            parent[j] = i;
            referenced[j] = true;
            children[i].push(j);
        }
    }

    for (i, r) in records.iter().enumerate() {
        if referenced[i] && r.version.as_deref().is_some_and(|v| !v.is_empty()) {
            return Err(Error::Validation(ValidationError::VersionOnSubtask(
                r.title.clone(),
            )));
        }
    }

    // Every task descends from exactly one root, so building the trees from
    // the unreferenced records walks the whole document — unless something is
    // in a cycle, which no root leads to. `seen` catches both that and a
    // back-edge reachable from a root.
    let mut seen = vec![false; n];
    let mut roots = Vec::new();
    for (i, is_root) in referenced.iter().enumerate() {
        if !*is_root {
            roots.push(build_tree(&records, &children, i, 0, &mut seen)?);
        }
    }
    if seen.iter().any(|seen| !*seen) {
        return Err(Error::Validation(ValidationError::CyclicReference));
    }
    Ok(roots)
}

#[cfg_attr(not(any(feature = "jsonl", feature = "csv")), allow(dead_code))]
fn build_tree(
    records: &[FlatTask],
    children: &[Vec<usize>],
    i: usize,
    depth: usize,
    seen: &mut [bool],
) -> Result<Task, Error> {
    check(Limit::Depth, depth).map_err(Error::Validation)?;
    if seen[i] {
        return Err(Error::Validation(ValidationError::CyclicReference));
    }
    seen[i] = true;
    let r = &records[i];
    let mut task = Task {
        title: r.title.clone(),
        status: r.status,
        id: r.id.clone(),
        version: r.version.clone(),
        notes: r.notes.clone(),
        metadata: r.metadata.clone(),
        ..Default::default()
    };
    for &j in &children[i] {
        task.subtasks
            .push(build_tree(records, children, j, depth + 1, seen)?);
    }
    Ok(task)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;
    use crate::v0::model::status::Status;

    fn parent(status: Status, child: Status) -> Vec<Task> {
        vec![Task {
            title: "parent".into(),
            status,
            subtasks: vec![Task::new("child".into(), child)],
            ..Default::default()
        }]
    }

    #[test]
    fn a_done_task_may_not_have_an_open_subtask() {
        let err = validate_forest(&parent(Status::Done, Status::Open)).unwrap_err();
        assert_eq!(err.code(), "open-subtask");
    }

    #[test]
    fn a_done_task_with_done_subtasks_is_fine() {
        assert!(validate_forest(&parent(Status::Done, Status::Done)).is_ok());
    }

    #[test]
    fn an_open_task_may_have_any_subtask_status() {
        assert!(validate_forest(&parent(Status::Open, Status::Done)).is_ok());
        assert!(validate_forest(&parent(Status::Open, Status::Open)).is_ok());
    }
}
