//! Lenient document shape: fields are read as `serde_json::Value` so a
//! missing or mistyped field is a coded rejection instead of a raw serde
//! message. Graph semantics stay in `helper`.
//!
//! One raw struct covers both families. Whether `subtasks` holds nested
//! tasks (tree formats) or child ids (flat formats) is the caller's
//! decision — the serialization, not the value, decides.

use crate::error::{Error, ValidationError};
use crate::model::flat_task::FlatTask;
use crate::model::status::Status;
use crate::model::task::Task;
use serde::Deserialize;
use serde_json::{Map, Value};

#[derive(Deserialize)]
pub(crate) struct RawTask {
    title: Option<Value>,
    status: Option<Value>,
    id: Option<Value>,
    version: Option<Value>,
    notes: Option<Value>,
    metadata: Option<Value>,
    subtasks: Option<Value>,
    #[serde(flatten)]
    extra: Map<String, Value>,
}

/// The fields every task shares, already type-checked.
struct Common {
    title: String,
    status: Status,
    id: Option<String>,
    version: Option<String>,
    notes: Option<String>,
    metadata: Option<Map<String, Value>>,
}

fn wrong_type(field: &'static str, expected: &'static str) -> Error {
    Error::Validation(ValidationError::InvalidFieldType { field, expected })
}

fn string_field(value: &Value, field: &'static str) -> Result<String, Error> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| wrong_type(field, "a string"))
}

fn status_from(value: &Value) -> Result<Status, Error> {
    match value.as_str() {
        Some("open") => Ok(Status::Open),
        Some("done") => Ok(Status::Done),
        Some(other) => Err(Error::Validation(ValidationError::InvalidStatus(
            other.to_owned(),
        ))),
        None => Err(wrong_type("status", "\"open\" or \"done\"")),
    }
}

fn metadata_from(value: Option<&Value>) -> Result<Option<Map<String, Value>>, Error> {
    match value {
        None => Ok(None),
        Some(Value::Object(map)) => Ok(Some(map.clone())),
        Some(_) => Err(wrong_type("metadata", "an object")),
    }
}

fn check_unknown(extra: &Map<String, Value>) -> Result<(), Error> {
    match extra.keys().next() {
        None => Ok(()),
        Some(key) => Err(Error::Validation(ValidationError::UnexpectedField(
            key.clone(),
        ))),
    }
}

fn common(raw: &RawTask) -> Result<Common, Error> {
    check_unknown(&raw.extra)?;
    let title = match raw.title {
        None => return Err(Error::Validation(ValidationError::MissingTitle)),
        Some(ref value) => string_field(value, "title")?,
    };
    let status = match raw.status {
        None => return Err(Error::Validation(ValidationError::MissingStatus)),
        Some(ref value) => status_from(value)?,
    };
    let id = raw.id.as_ref().map(|v| string_field(v, "id")).transpose()?;
    let version = raw
        .version
        .as_ref()
        .map(|v| string_field(v, "version"))
        .transpose()?;
    let notes = raw
        .notes
        .as_ref()
        .map(|v| string_field(v, "notes"))
        .transpose()?;
    Ok(Common {
        title,
        status,
        id,
        version,
        notes,
        metadata: metadata_from(raw.metadata.as_ref())?,
    })
}

fn child(value: &Value) -> Result<RawTask, Error> {
    serde_json::from_value(value.clone()).map_err(|e| Error::Json(e.to_string()))
}

/// Build one tree task from its lenient raw form (nested `subtasks`).
pub(crate) fn build_task(raw: RawTask) -> Result<Task, Error> {
    let common = common(&raw)?;
    let subtasks = match &raw.subtasks {
        None => Vec::new(),
        Some(Value::Array(items)) => items
            .iter()
            .map(|value| build_task(child(value)?))
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

/// Build one flat record from its lenient raw form (`subtasks` is a list
/// of child ids).
pub(crate) fn build_flat_task(raw: RawTask) -> Result<FlatTask, Error> {
    let common = common(&raw)?;
    let subtasks = match &raw.subtasks {
        None => None,
        Some(Value::Array(items)) => Some(
            items
                .iter()
                .map(|item| string_field(item, "subtask id"))
                .collect::<Result<Vec<String>, Error>>()?,
        ),
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
            build_task(raw(json!({"status": "open"})))
                .unwrap_err()
                .code(),
            "missing-title"
        );
        assert_eq!(
            build_task(raw(json!({"title": "t"}))).unwrap_err().code(),
            "missing-status"
        );
    }

    #[test]
    fn wrong_types_are_coded() {
        assert_eq!(
            build_task(raw(json!({"title": 1, "status": "open"})))
                .unwrap_err()
                .code(),
            "invalid-field-type"
        );
        assert_eq!(
            build_task(raw(json!({"title": "t", "status": "nope"})))
                .unwrap_err()
                .code(),
            "invalid-status"
        );
        assert_eq!(
            build_task(raw(json!({"title": "t", "status": "open", "metadata": []})))
                .unwrap_err()
                .code(),
            "invalid-field-type"
        );
        assert_eq!(
            build_task(raw(json!({"title": "t", "status": "open", "subtasks": {}})))
                .unwrap_err()
                .code(),
            "invalid-field-type"
        );
    }

    #[test]
    fn unknown_fields_are_coded() {
        assert_eq!(
            build_task(raw(
                json!({"title": "t", "status": "open", "colour": "red"})
            ))
            .unwrap_err()
            .code(),
            "unexpected-field"
        );
    }

    #[test]
    fn a_valid_tree_task_builds() {
        let task = build_task(raw(json!({
            "title": "t",
            "status": "open",
            "metadata": {"tags": ["a"]},
            "subtasks": [{"title": "c", "status": "done"}]
        })))
        .unwrap();
        assert_eq!(task.subtasks.len(), 1);
        assert_eq!(task.subtasks[0].status, Status::Done);
    }

    #[test]
    fn a_valid_flat_record_builds() {
        let record = build_flat_task(raw(json!({
            "id": "root",
            "title": "t",
            "status": "open",
            "subtasks": ["a", "b"]
        })))
        .unwrap();
        assert_eq!(record.child_ids(), ["a", "b"]);
    }
}
