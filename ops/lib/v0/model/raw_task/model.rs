//! The lenient wire shape: fields are read as `serde_json::Value` so a
//! missing or mistyped field is a coded rejection instead of a raw serde
//! message.
//!
//! One raw struct covers both families. Whether `subtasks` holds nested
//! tasks (tree formats) or child ids (flat formats) is the caller's
//! decision — the serialization, not the value, decides.

use super::helpers::{child, string_field, wrong_type};
use super::validators::common;
use crate::v0::error::Error;
use crate::v0::model::flat_task::FlatTask;
use crate::v0::model::status::Status;
use crate::v0::model::task::Task;
use serde::Deserialize;
use serde_json::{Map, Value};

/// One task as it appears on the wire.
#[derive(Deserialize)]
pub(crate) struct RawTask {
    pub(super) title: Option<Value>,
    pub(super) status: Option<Value>,
    pub(super) id: Option<Value>,
    pub(super) version: Option<Value>,
    pub(super) notes: Option<Value>,
    pub(super) metadata: Option<Value>,
    pub(super) subtasks: Option<Value>,
    #[serde(flatten)]
    pub(super) extra: Map<String, Value>,
}

/// The fields every task shares, already type-checked.
pub(super) struct Common {
    pub(super) title: String,
    pub(super) status: Status,
    pub(super) id: Option<String>,
    pub(super) version: Option<String>,
    pub(super) notes: Option<String>,
    pub(super) metadata: Option<Map<String, Value>>,
}

impl RawTask {
    /// Convert one tree task from its lenient raw form (nested `subtasks`).
    pub(crate) fn to_task(&self) -> Result<Task, Error> {
        let common = common(self)?;
        let subtasks = match &self.subtasks {
            None => Vec::new(),
            Some(Value::Array(items)) => items
                .iter()
                .map(|value| child(value)?.to_task())
                .collect::<Result<Vec<Task>, Error>>()?,
            Some(_) => return Err(wrong_type("subtasks", "an array")),
        };
        Ok(Task {
            title: common.title,
            status: common.status,
            id: common.id,
            version: common.version,
            notes: common.notes,
            metadata: common.metadata,
            subtasks,
        })
    }

    /// Convert one flat record from its lenient raw form (`subtasks` is a
    /// list of child ids).
    pub(crate) fn to_flat_task(&self) -> Result<FlatTask, Error> {
        let common = common(self)?;
        let subtasks = match &self.subtasks {
            None => Vec::new(),
            Some(Value::Array(items)) => items
                .iter()
                .map(|item| string_field(item, "subtask id"))
                .collect::<Result<Vec<String>, Error>>()?,
            Some(_) => return Err(wrong_type("subtasks", "an array of ids")),
        };
        Ok(FlatTask {
            title: common.title,
            status: common.status,
            id: common.id,
            version: common.version,
            notes: common.notes,
            metadata: common.metadata,
            subtasks,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;
    use serde_json::json;

    fn raw(value: Value) -> RawTask {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn missing_fields_are_coded() {
        assert_eq!(
            raw(json!({"status": "open"})).to_task().unwrap_err().code(),
            "missing-title"
        );
        assert_eq!(
            raw(json!({"title": "t"})).to_task().unwrap_err().code(),
            "missing-status"
        );
    }

    #[test]
    fn wrong_types_are_coded() {
        assert_eq!(
            raw(json!({"title": 1, "status": "open"}))
                .to_task()
                .unwrap_err()
                .code(),
            "invalid-field-type"
        );
        assert_eq!(
            raw(json!({"title": "t", "status": "nope"}))
                .to_task()
                .unwrap_err()
                .code(),
            "invalid-status"
        );
        assert_eq!(
            raw(json!({"title": "t", "status": "open", "metadata": []}))
                .to_task()
                .unwrap_err()
                .code(),
            "invalid-field-type"
        );
        assert_eq!(
            raw(json!({"title": "t", "status": "open", "subtasks": {}}))
                .to_task()
                .unwrap_err()
                .code(),
            "invalid-field-type"
        );
    }

    #[test]
    fn unknown_fields_are_coded() {
        assert_eq!(
            raw(json!({"title": "t", "status": "open", "colour": "red"}))
                .to_task()
                .unwrap_err()
                .code(),
            "unexpected-field"
        );
    }

    #[test]
    fn a_valid_tree_task_builds() {
        let task = raw(json!({
            "title": "t",
            "status": "open",
            "metadata": {"tags": ["a"]},
            "subtasks": [{"title": "c", "status": "done"}]
        }))
        .to_task()
        .unwrap();
        assert_eq!(task.subtasks.len(), 1);
        assert_eq!(task.subtasks[0].status, Status::Done);
    }

    #[test]
    fn a_valid_flat_record_builds() {
        let record = raw(json!({
            "id": "root",
            "title": "t",
            "status": "open",
            "subtasks": ["a", "b"]
        }))
        .to_flat_task()
        .unwrap();
        assert_eq!(record.child_ids(), ["a", "b"]);
    }
}
