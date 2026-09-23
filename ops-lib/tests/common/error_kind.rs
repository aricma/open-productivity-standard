use ops_lib::{Error, ValidationError};

/// Error classes worth asserting across read/write cases. Payloads are
/// ignored except for `Validation`, whose payload is the semantic
/// contract.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    Validation(ValidationError),
    SingleRoot,
    Json,
    Yaml,
}

impl From<&Error> for ErrorKind {
    fn from(e: &Error) -> Self {
        match e {
            Error::Validation(v) => ErrorKind::Validation(v.clone()),
            Error::SingleRootExpected(_) => ErrorKind::SingleRoot,
            Error::Json(_) => ErrorKind::Json,
            Error::Yaml(_) => ErrorKind::Yaml,
        }
    }
}
