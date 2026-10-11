//! The one reader the tree converters still need here.

use super::model::RawTask;
use crate::v0::error::Error;
use crate::v0::model::fields::wrong_type;
use serde_json::Value;

/// One entry of a tree-format `subtasks` list, which must be an object.
#[cfg_attr(not(any(feature = "json", feature = "yaml")), allow(dead_code))]
pub(super) fn child(value: &Value) -> Result<RawTask, Error> {
    match value {
        Value::Object(fields) => RawTask::from_fields(fields.clone()),
        _ => Err(wrong_type("subtasks", "an array of task objects")),
    }
}
