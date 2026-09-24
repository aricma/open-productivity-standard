//! Small readers shared by the validators and the converters.

use super::model::RawTask;
use crate::v0::error::{Error, ValidationError};
use serde_json::Value;

pub(super) fn wrong_type(field: &'static str, expected: &'static str) -> Error {
    Error::Validation(ValidationError::InvalidFieldType { field, expected })
}

pub(super) fn string_field(value: &Value, field: &'static str) -> Result<String, Error> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| wrong_type(field, "a string"))
}

pub(super) fn child(value: &Value) -> Result<RawTask, Error> {
    serde_json::from_value(value.clone()).map_err(|e| Error::Json(e.to_string()))
}
