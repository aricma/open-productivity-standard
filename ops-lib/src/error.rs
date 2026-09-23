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
                crate::config::RELEASED_VERSIONS.join(", ")
            ),
            ValidationError::InvalidMetadataKey(key) => {
                write!(f, "metadata key `{key}` breaks ^[a-z0-9_]{{3,}}$")
            }
        }
    }
}

impl std::error::Error for ValidationError {}
