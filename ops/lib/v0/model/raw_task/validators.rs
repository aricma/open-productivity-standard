//! Field validation: every raw field must have the right shape, or the
//! document is rejected with a coded error.

use super::model::{Common, RawTask};
use crate::v0::error::{Error, ValidationError};
use crate::v0::model::fields::{status_from, string_field, wrong_type};
use crate::v0::model::value::{Metadata, MetadataValue};
use serde_json::{Map, Value};

fn metadata_from(value: Option<&Value>) -> Result<Option<Metadata>, Error> {
    match value {
        None => Ok(None),
        Some(Value::Object(map)) => {
            let mut out = Metadata::from_json_map(map.clone());
            out.retain(|_, v| !empty_value(v));
            Ok(if out.is_empty() { None } else { Some(out) })
        }
        Some(_) => Err(wrong_type("metadata", "an object")),
    }
}

/// A value that carries nothing: `null`, or an empty string, list, or map.
fn empty_value(v: &MetadataValue) -> bool {
    match v {
        MetadataValue::Null => true,
        MetadataValue::String(s) => s.is_empty(),
        MetadataValue::List(a) => a.is_empty(),
        MetadataValue::Map(o) => o.is_empty(),
        _ => false,
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
