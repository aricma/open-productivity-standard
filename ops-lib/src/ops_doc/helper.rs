//! Shared helpers for the serialization implementations: the semantic
//! validation gate every parser runs, plus the flat-format machinery
//! (records are one task per line/row, hierarchy is expressed by
//! child-id lists, and the tree is rebuilt by resolving them — shared
//! children, missing references, and cycles are rejected here, since a
//! flat document with a broken graph cannot even be represented as a
//! tree).

use crate::error::{Error, ValidationError};
use crate::ops_model::flat_task::FlatTask;
use crate::ops_model::task::Task;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

#[allow(clippy::expect_used)]
static METADATA_KEY_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9_]{3,}$").expect("static pattern is valid"));

fn valid_metadata_key(key: &str) -> bool {
    METADATA_KEY_PATTERN.is_match(key)
}

fn walk(task: &Task, depth: usize, ids: &mut HashSet<String>) -> Result<(), ValidationError> {
    if let Some(metadata) = &task.metadata {
        for key in metadata.keys() {
            if !valid_metadata_key(key) {
                return Err(ValidationError::InvalidMetadataKey(key.clone()));
            }
        }
    }
    if let Some(id) = &task.id
        && !ids.insert(id.clone())
    {
        return Err(ValidationError::DuplicateId(id.clone()));
    }
    if depth > 0 && task.version.is_some() {
        return Err(ValidationError::VersionOnSubtask(task.title.clone()));
    }
    for child in &task.subtasks {
        walk(child, depth + 1, ids)?;
    }
    Ok(())
}

/// Validate an in-memory forest against the semantic rules.
pub(crate) fn validate_forest(tasks: &[Task]) -> Result<(), ValidationError> {
    let mut ids = HashSet::new();
    for t in tasks {
        walk(t, 0, &mut ids)?;
    }
    Ok(())
}

/// Semantic gate used by the format parsers: parsing never returns an
/// invalid forest.
pub(crate) fn validated_tasks(tasks: Vec<Task>) -> Result<Vec<Task>, Error> {
    validate_forest(&tasks).map_err(Error::Validation)?;
    Ok(tasks)
}

/// Flatten a task tree into records (pre-order, root first).
/// Flatten a forest into records (shared by the JSONL and CSV writers).
pub(crate) fn records(tasks: &[Task]) -> Vec<FlatTask> {
    let mut out = Vec::new();
    for t in tasks {
        flatten(t, &mut out);
    }
    out
}

pub fn flatten(task: &Task, out: &mut Vec<FlatTask>) {
    out.push(FlatTask::from_task(task));
    for child in &task.subtasks {
        flatten(child, out);
    }
}

/// Rebuild a forest from flat records, enforcing the graph rules:
/// unique ids, every referenced id exists, one parent per task, no
/// cycles (a collection with no root is a cycle), `version` on roots
/// only.
pub fn forest(records: Vec<FlatTask>) -> Result<Vec<Task>, Error> {
    let n = records.len();
    let mut by_id: HashMap<String, usize> = HashMap::new();
    for (i, r) in records.iter().enumerate() {
        if let Some(id) = &r.id
            && by_id.insert(id.clone(), i).is_some()
        {
            return Err(Error::Validation(ValidationError::DuplicateId(id.clone())));
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
        if referenced[i] && r.version.is_some() {
            return Err(Error::Validation(ValidationError::VersionOnSubtask(
                r.title.clone(),
            )));
        }
    }

    // Every task descends from exactly one root; a fully referenced
    // collection is a cycle (e.g. a ↔ b).
    let mut roots = Vec::new();
    for (i, is_root) in referenced.iter().enumerate() {
        if !*is_root {
            roots.push(build_tree(&records, &children, i)?);
        }
    }
    if !records.is_empty() && roots.is_empty() {
        return Err(Error::Validation(ValidationError::CyclicReference));
    }
    Ok(roots)
}

fn build_tree(records: &[FlatTask], children: &[Vec<usize>], i: usize) -> Result<Task, Error> {
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
        task.subtasks.push(build_tree(records, children, j)?);
    }
    Ok(task)
}
