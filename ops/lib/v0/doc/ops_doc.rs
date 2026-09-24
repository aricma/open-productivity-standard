//! The serialization trait implemented by every format module.

use crate::v0::error::Error;
use crate::v0::model::raw_task::RawTask;
use crate::v0::model::task::Task;

/// A serialization format: its own syntax in, raw tasks out, and the
/// model back to syntax.
///
/// A format module does syntax only. `parse` reads its syntax into raw
/// tasks — no model, no validation; the `read`/`write` facade builds the
/// model from those raw tasks and validates it.
pub(crate) trait OpsDoc {
    /// This format's syntax into raw tasks: one for a tree document, one
    /// per record for a flat stream.
    fn parse(self, input: &str) -> Result<Vec<RawTask>, Error>;

    /// The model into this format's syntax. Tree formats require exactly
    /// one root; flat formats accept any forest, including empty.
    fn write(self, tasks: &[Task]) -> Result<String, Error>;
}
