//! Small field readers shared by the model validators and the format
//! modules: one place for "this field must be a string" and "this field
//! must be a status".

use crate::v0::error::{Error, ValidationError};
use crate::v0::model::status::Status;
use serde_json::Value;

pub(crate) fn wrong_type(field: &'static str, expected: &'static str) -> Error {
    Error::Validation(ValidationError::InvalidFieldType { field, expected })
}

pub(crate) fn string_field(value: &Value, field: &'static str) -> Result<String, Error> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| wrong_type(field, "a string"))
}

pub(crate) fn status_from(value: &Value) -> Result<Status, Error> {
    match value.as_str() {
        Some("open") => Ok(Status::Open),
        Some("done") => Ok(Status::Done),
        Some(other) => Err(Error::Validation(ValidationError::InvalidStatus(
            other.to_owned(),
        ))),
        None => Err(wrong_type("status", "\"open\" or \"done\"")),
    }
}
