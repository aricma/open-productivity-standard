use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Validation(ValidationError),
    SingleRootExpected(String),
    Json(String),
    Yaml(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Validation(e) => write!(f, "invalid OPS document: {e}"),
            Error::SingleRootExpected(fmt) => {
                write!(f, "format `{fmt}` holds exactly one root task")
            }
            Error::Json(e) => write!(f, "invalid JSON: {e}"),
            Error::Yaml(e) => write!(f, "invalid YAML: {e}"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    DuplicateId(String),
    MissingReference(String),
    SharedChild(String),
    CyclicReference,
    VersionOnSubtask(String),
    UnknownOPSVersion(String),
    InvalidMetadataKey(String),
    MissingTitle,
    MissingStatus,
    InvalidStatus(String),
    UnexpectedField(String),
    InvalidFieldType {
        field: &'static str,
        expected: &'static str,
    },
    DuplicateMetadataKey(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::DuplicateId(id) => write!(f, "duplicate task id `{id}`"),
            ValidationError::MissingReference(id) => write!(f, "subtask id `{id}` does not exist"),
            ValidationError::SharedChild(id) => {
                write!(f, "task id `{id}` has more than one parent")
            }
            ValidationError::CyclicReference => {
                write!(f, "references form a cycle; the document has no root")
            }
            ValidationError::VersionOnSubtask(title) => {
                write!(f, "task `{title}` carries a version below a root")
            }
            ValidationError::UnknownOPSVersion(v) => write!(
                f,
                "version `{v}` is not a released OPS version (known: {})",
                crate::shared::version::RELEASED_VERSIONS.join(", ")
            ),
            ValidationError::InvalidMetadataKey(key) => {
                write!(f, "metadata key `{key}` breaks ^[a-z0-9_]{{3,}}$")
            }
            ValidationError::MissingTitle => write!(f, "a task has no title"),
            ValidationError::MissingStatus => write!(f, "a task has no status"),
            ValidationError::InvalidStatus(s) => {
                write!(f, "status `{s}` is not `open` or `done`")
            }
            ValidationError::UnexpectedField(field) => {
                write!(f, "field `{field}` is not part of the model")
            }
            ValidationError::InvalidFieldType { field, expected } => {
                write!(f, "`{field}` must be {expected}")
            }
            ValidationError::DuplicateMetadataKey(key) => {
                write!(f, "metadata key `{key}` is supplied twice")
            }
        }
    }
}

impl std::error::Error for ValidationError {}

impl ValidationError {
    /// The library's stable code for this violation, for diagnostics and
    /// tests. This is an ops-lib convenience, not part of the OPS
    /// specification.
    pub fn code(&self) -> &'static str {
        match self {
            Self::DuplicateId(_) => "duplicate-id",
            Self::MissingReference(_) => "missing-reference",
            Self::SharedChild(_) => "shared-child",
            Self::CyclicReference => "cyclic-reference",
            Self::VersionOnSubtask(_) => "version-on-subtask",
            Self::UnknownOPSVersion(_) => "unknown-version",
            Self::InvalidMetadataKey(_) => "invalid-metadata-key",
            Self::MissingTitle => "missing-title",
            Self::MissingStatus => "missing-status",
            Self::InvalidStatus(_) => "invalid-status",
            Self::UnexpectedField(_) => "unexpected-field",
            Self::InvalidFieldType { .. } => "invalid-field-type",
            Self::DuplicateMetadataKey(_) => "duplicate-metadata-key",
        }
    }
}

impl Error {
    /// The library's stable code for this rejection, for diagnostics and
    /// tests. This is an ops-lib convenience, not part of the OPS
    /// specification.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(v) => v.code(),
            Self::SingleRootExpected(_) => "single-root-expected",
            Self::Json(_) | Self::Yaml(_) => "invalid-syntax",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_errors_carry_stable_codes() {
        assert_eq!(ValidationError::MissingTitle.code(), "missing-title");
        assert_eq!(ValidationError::MissingStatus.code(), "missing-status");
        assert_eq!(
            ValidationError::InvalidStatus("nope".into()).code(),
            "invalid-status"
        );
        assert_eq!(
            ValidationError::UnexpectedField("extra".into()).code(),
            "unexpected-field"
        );
        assert_eq!(
            ValidationError::InvalidFieldType {
                field: "metadata",
                expected: "an object"
            }
            .code(),
            "invalid-field-type"
        );
        assert_eq!(
            ValidationError::DuplicateMetadataKey("tags".into()).code(),
            "duplicate-metadata-key"
        );
        assert_eq!(
            ValidationError::DuplicateId("a".into()).code(),
            "duplicate-id"
        );
        assert_eq!(
            ValidationError::MissingReference("a".into()).code(),
            "missing-reference"
        );
        assert_eq!(
            ValidationError::SharedChild("a".into()).code(),
            "shared-child"
        );
        assert_eq!(ValidationError::CyclicReference.code(), "cyclic-reference");
        assert_eq!(
            ValidationError::VersionOnSubtask("t".into()).code(),
            "version-on-subtask"
        );
        assert_eq!(
            ValidationError::UnknownOPSVersion("9".into()).code(),
            "unknown-version"
        );
        assert_eq!(
            ValidationError::InvalidMetadataKey("A".into()).code(),
            "invalid-metadata-key"
        );
    }

    #[test]
    fn errors_delegate_to_their_code() {
        assert_eq!(
            Error::Validation(ValidationError::MissingTitle).code(),
            "missing-title"
        );
        assert_eq!(Error::Json("bad".into()).code(), "invalid-syntax");
        assert_eq!(Error::Yaml("bad".into()).code(), "invalid-syntax");
        assert_eq!(
            Error::SingleRootExpected("json".into()).code(),
            "single-root-expected"
        );
    }
}
