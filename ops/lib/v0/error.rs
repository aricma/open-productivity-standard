use crate::v0::format::Format;
use crate::v0::limits::Limit;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Validation(ValidationError),
    /// One format's syntax, or a value shaped unlike a task.
    Syntax {
        format: Format,
        message: String,
    },
}

impl Error {
    /// A syntax or shape rejection from one format.
    pub(crate) fn syntax(format: Format, message: impl ToString) -> Error {
        Error::Syntax {
            format,
            message: message.to_string(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Validation(e) => write!(f, "invalid OPS document: {e}"),
            Error::Syntax { format, message } => {
                write!(f, "invalid {}: {message}", format.display_name())
            }
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
    UnsupportedVersion(String),
    MixedVersions {
        first: String,
        second: String,
    },
    InvalidMetadataKey(String),
    MissingTitle,
    MissingStatus,
    InvalidStatus(String),
    EmptyTitle,
    TitleHasLineBreak,
    UnexpectedField(String),
    InvalidFieldType {
        field: &'static str,
        expected: &'static str,
    },
    DuplicateMetadataKey(String),
    OpenSubtask {
        task: String,
        subtask: String,
    },
    SubtaskWithoutId {
        task: String,
        subtask: String,
    },
    LimitExceeded {
        limit: Limit,
        got: usize,
    },
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
                write!(
                    f,
                    "references form a cycle; a task cannot descend from itself"
                )
            }
            ValidationError::VersionOnSubtask(title) => {
                write!(f, "task `{title}` carries a version below a root")
            }
            ValidationError::UnsupportedVersion(v) => write!(
                f,
                "this library does not implement version `{v}` (it implements: {})",
                crate::shared::version::SUPPORTED_VERSIONS.join(", ")
            ),
            ValidationError::MixedVersions { first, second } => {
                write!(f, "document declares both version `{first}` and `{second}`")
            }
            ValidationError::InvalidMetadataKey(key) => {
                write!(f, "metadata key `{key}` breaks ^[a-z0-9_]{{3,}}$")
            }
            ValidationError::MissingTitle => write!(f, "a task has no title"),
            ValidationError::MissingStatus => write!(f, "a task has no status"),
            ValidationError::EmptyTitle => write!(f, "a task has an empty title"),
            ValidationError::TitleHasLineBreak => {
                write!(f, "a task's title contains a line break")
            }
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
            ValidationError::OpenSubtask { task, subtask } => {
                write!(f, "task `{task}` is done but subtask `{subtask}` is open")
            }
            ValidationError::SubtaskWithoutId { task, subtask } => write!(
                f,
                "subtask `{subtask}` of task `{task}` has no id, which a flat format needs"
            ),
            ValidationError::LimitExceeded { limit, got } => write!(
                f,
                "{} is {got} {}; the limit is {}",
                limit.name(),
                limit.unit(),
                limit.max()
            ),
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
            Self::UnsupportedVersion(_) => "unsupported-version",
            Self::MixedVersions { .. } => "mixed-versions",
            Self::InvalidMetadataKey(_) => "invalid-metadata-key",
            Self::MissingTitle => "missing-title",
            Self::MissingStatus => "missing-status",
            Self::EmptyTitle => "empty-title",
            Self::TitleHasLineBreak => "title-has-line-break",
            Self::InvalidStatus(_) => "invalid-status",
            Self::UnexpectedField(_) => "unexpected-field",
            Self::InvalidFieldType { .. } => "invalid-field-type",
            Self::DuplicateMetadataKey(_) => "duplicate-metadata-key",
            Self::OpenSubtask { .. } => "open-subtask",
            Self::SubtaskWithoutId { .. } => "subtask-without-id",
            Self::LimitExceeded { limit, .. } => limit.code(),
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
            Self::Syntax { .. } => "invalid-syntax",
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
        assert_eq!(ValidationError::EmptyTitle.code(), "empty-title");
        assert_eq!(
            ValidationError::TitleHasLineBreak.code(),
            "title-has-line-break"
        );
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
            ValidationError::OpenSubtask {
                task: "p".into(),
                subtask: "c".into(),
            }
            .code(),
            "open-subtask"
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
            ValidationError::SubtaskWithoutId {
                task: "p".into(),
                subtask: "c".into(),
            }
            .code(),
            "subtask-without-id"
        );
        assert_eq!(
            ValidationError::UnsupportedVersion("9".into()).code(),
            "unsupported-version"
        );
        assert_eq!(
            ValidationError::MixedVersions {
                first: "0".into(),
                second: "1".into(),
            }
            .code(),
            "mixed-versions"
        );
        assert_eq!(
            ValidationError::InvalidMetadataKey("A".into()).code(),
            "invalid-metadata-key"
        );
        assert_eq!(
            ValidationError::LimitExceeded {
                limit: crate::v0::limits::Limit::Depth,
                got: 257,
            }
            .code(),
            "nesting-too-deep"
        );
    }

    #[test]
    fn syntax_errors_share_one_code() {
        for &format in Format::ALL {
            let error = Error::syntax(format, "x");
            assert_eq!(error.code(), "invalid-syntax");
            assert!(
                error
                    .to_string()
                    .starts_with(&format!("invalid {}: ", format.display_name())),
                "{error}"
            );
        }
    }

    #[test]
    fn limit_errors_name_the_limit_and_the_value() {
        let err = ValidationError::LimitExceeded {
            limit: crate::v0::limits::Limit::Title,
            got: 1025,
        };
        assert_eq!(err.to_string(), "title is 1025 bytes; the limit is 1024");
    }

    #[test]
    fn errors_delegate_to_their_code() {
        assert_eq!(
            Error::Validation(ValidationError::MissingTitle).code(),
            "missing-title"
        );
        for &format in Format::ALL {
            assert_eq!(Error::syntax(format, "bad").code(), "invalid-syntax");
        }
    }
}
