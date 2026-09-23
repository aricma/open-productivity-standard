//! The [`OpsDoc`] trait, implemented by each serialization format in
//! its own module (`json`, `yaml`, `jsonl`).

pub(crate) mod helper;
pub(crate) mod json;
pub(crate) mod jsonl;
pub(crate) mod yaml;

use crate::error::Error;
use crate::ops_model::task::Task;

/// The OPS document trait: the lib's core capability, implemented by
/// every serialization format.
///
/// [`parse`](OpsDoc::parse) reads a document string into valid tasks —
/// it never returns an invalid forest; [`write`](OpsDoc::write)
/// serializes tasks back into a document string (tree formats require
/// exactly one root, flat formats accept any forest, including empty).
///
/// Validation is deliberately not part of this trait: it operates on
/// the in-memory forest and every parser runs it internally (the
/// `helper::validated` gate).
pub trait OpsDoc {
    fn parse(self, input: &str) -> Result<Vec<Task>, Error>;

    fn write(self, tasks: &[Task]) -> Result<String, Error>;
}

/// Tree-preserving formats hold exactly one root per document.
pub(crate) fn single_root<'a>(tasks: &'a [Task], format: &str) -> Result<&'a Task, Error> {
    match tasks {
        [t] => Ok(t),
        _ => Err(Error::SingleRootExpected(format.to_string())),
    }
}
