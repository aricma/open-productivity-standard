//! OPS version 0: the reader and writer for
//! [open_productivity_standard_v0](../../../specs/open_productivity_standard_v0.md).
//!
//! The task model (one task type, nested arbitrarily deep) and these
//! serializations:
//!
//! - tree-preserving: JSON, YAML, Markdown
//! - flat/streamable: JSONL, CSV
//!
//! Version 0 is the only version; a future version gets its own module
//! and does not disturb this one.

mod error;
mod format;
pub mod limits;
mod version;

pub(crate) mod doc;
pub(crate) mod model;

use doc::OpsDoc;
use doc::csv::Csv;
use doc::helper::{forest, validate_forest};
use doc::json::Json;
use doc::jsonl::Jsonl;
use doc::md::Md;
use doc::yaml::Yaml;
use limits::{Limit, check};
use model::raw_task::RawTask;

pub use error::{Error, ValidationError};
pub use format::Format;
pub use model::status::Status;
pub use model::task::Task;

/// Imports a version-0 OPS document in `format` into valid tasks.
pub fn read(format: Format, input: &str) -> Result<Vec<Task>, Error> {
    check(Limit::InputBytes, input.len()).map_err(Error::Validation)?;
    let raw = match format {
        Format::Json => Json.parse(input)?,
        Format::Yaml => Yaml.parse(input)?,
        Format::Jsonl => Jsonl.parse(input)?,
        Format::Csv => Csv.parse(input)?,
        Format::Md => Md.parse(input)?,
    };
    let tasks = match format {
        Format::Json | Format::Yaml | Format::Md => match raw.as_slice() {
            [task] => vec![task.to_task()?],
            _ => return Err(Error::SingleRootExpected(format.name().to_string())),
        },
        Format::Jsonl | Format::Csv => {
            check(Limit::Tasks, raw.len()).map_err(Error::Validation)?;
            forest(
                raw.iter()
                    .map(RawTask::to_flat_task)
                    .collect::<Result<Vec<_>, Error>>()?,
            )?
        }
    };
    validate_forest(&tasks).map_err(Error::Validation)?;
    version::check_released_versions(&tasks)?;
    Ok(tasks)
}

/// Exports tasks as a version-0 OPS document in `format`.
pub fn write(format: Format, tasks: &[Task]) -> Result<String, Error> {
    version::check_released_versions(tasks)?;
    validate_forest(tasks).map_err(Error::Validation)?;
    match format {
        Format::Json => Json.write(tasks),
        Format::Yaml => Yaml.write(tasks),
        Format::Jsonl => Jsonl.write(tasks),
        Format::Csv => Csv.write(tasks),
        Format::Md => Md.write(tasks),
    }
}
