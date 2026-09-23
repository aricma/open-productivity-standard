//! Lib-level helpers shared by the read/write facades.

use crate::config::RELEASED_VERSIONS;
use crate::error::{Error, ValidationError};
use crate::ops_model::task::Task;

pub(crate) fn validate_used_ops_version_against_official_releases(
    tasks: &[Task],
) -> Result<(), Error> {
    for t in tasks {
        if let Some(v) = &t.version
            && !RELEASED_VERSIONS.contains(&v.as_str())
        {
            return Err(Error::Validation(ValidationError::UnknownOPSVersion(
                v.clone(),
            )));
        }
    }
    Ok(())
}
