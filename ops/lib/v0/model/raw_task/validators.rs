//! Field validation: every raw field must have the right shape, or the
//! document is rejected with a coded error.

use super::helpers::{string_field, wrong_type};
use super::model::{Common, RawTask};
use crate::v0::error::{Error, ValidationError};
use crate::v0::model::status::Status;
use serde_json::{Map, Value};

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

/// Validates the fields every task shares and returns them type-checked.
pub(super) fn common(raw: &RawTask) -> Result<Common, Error> {
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
