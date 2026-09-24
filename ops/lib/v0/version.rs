//! Version-0 application of the shared released-version rule: every
//! `version` a document declares must name an official release.

use crate::shared::version;
use crate::v0::error::{Error, ValidationError};
use crate::v0::model::task::Task;

/// Rejects the first task whose `version` is not an official release.
pub(crate) fn check_released_versions(tasks: &[Task]) -> Result<(), Error> {
    match version::first_unreleased(tasks.iter().filter_map(|t| t.version.as_deref())) {
        Some(v) => Err(Error::Validation(ValidationError::UnknownOPSVersion(v))),
        None => Ok(()),
    }
}
