//! OPS version 0: the reader and writer for
//! [the version-0 model](../../../specs/v0/model.md).
//!
//! The task model (one task type, nested arbitrarily deep) and these
//! formats:
//!
//! - tree-preserving: JSON, YAML
//! - flat/streamable: JSONL, CSV
//!
//! Version 0 is the only version; a future version gets its own module
//! and does not disturb this one.

mod detect;
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
use doc::yaml::Yaml;
use limits::{Limit, check};
use model::raw_task::RawTask;

pub use error::{Error, ValidationError};
pub use format::Format;
pub use model::status::Status;
pub use model::task::Task;
pub use model::value::{Metadata, MetadataValue, Number};

/// Imports a version-0 OPS document in `format` into valid tasks.
pub fn read(format: Format, input: &str) -> Result<Vec<Task>, Error> {
    check(Limit::InputBytes, input.len()).map_err(Error::Validation)?;
    let raw = match format {
        Format::Json => Json.parse(input)?,
        Format::Yaml => Yaml.parse(input)?,
        Format::Jsonl => Jsonl.parse(input)?,
        Format::Csv => Csv.parse(input)?,
    };
    let tasks = match format {
        Format::Json | Format::Yaml => raw
            .iter()
            .map(RawTask::to_task)
            .collect::<Result<Vec<_>, Error>>()?,
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
    version::check_document_versions(&tasks)?;
    Ok(tasks)
}

/// The version this document declares, or `None` for the latest release.
/// Version-agnostic: it never calls a version module.
pub fn detect_version(format: Format, input: &str) -> Result<Option<String>, Error> {
    detect::detect_version(format, input)
}

/// Exports tasks as a version-0 OPS document in `format`.
pub fn write(format: Format, tasks: &[Task]) -> Result<String, Error> {
    version::check_document_versions(tasks)?;
    validate_forest(tasks).map_err(Error::Validation)?;
    match format {
        Format::Json => Json.write(tasks),
        Format::Yaml => Yaml.write(tasks),
        Format::Jsonl => Jsonl.write(tasks),
        Format::Csv => Csv.write(tasks),
    }
}
