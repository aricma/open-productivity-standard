//! Version-0 application of the library's supported-version policy, plus the
//! version rules: a document declares at most one version, and every
//! declaration must agree.

use crate::shared::version;
use crate::v0::error::{Error, ValidationError};
use crate::v0::model::task::Task;

/// Enforces the version rules: one version per document. Every declared
/// version must be identical and an official release.
pub(crate) fn check_document_versions(tasks: &[Task]) -> Result<(), Error> {
    let mut declared: Vec<&str> = tasks.iter().filter_map(|t| t.version.as_deref()).collect();
    declared.sort_unstable();
    declared.dedup();
    if let [first, second, ..] = declared.as_slice() {
        return Err(Error::Validation(ValidationError::MixedVersions {
            first: (*first).to_owned(),
            second: (*second).to_owned(),
        }));
    }
    match version::first_unsupported(declared) {
        Some(v) => Err(Error::Validation(ValidationError::UnsupportedVersion(v))),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;
    use crate::v0::model::status::Status;

    fn root(version: Option<&str>) -> Task {
        Task {
            version: version.map(str::to_owned),
            ..Task::new("t".into(), Status::Open)
        }
    }

    #[test]
    fn one_declared_version_is_allowed() {
        assert!(check_document_versions(&[root(Some("0")), root(Some("0"))]).is_ok());
        assert!(check_document_versions(&[root(None), root(Some("0"))]).is_ok());
        assert!(check_document_versions(&[]).is_ok());
    }

    #[test]
    fn two_declared_versions_are_mixed() {
        let err = check_document_versions(&[root(Some("0")), root(Some("1"))]).unwrap_err();
        assert_eq!(err.code(), "mixed-versions");
    }

    #[test]
    fn an_unsupported_version_is_rejected() {
        let err = check_document_versions(&[root(Some("1"))]).unwrap_err();
        assert_eq!(err.code(), "unsupported-version");
    }
}
