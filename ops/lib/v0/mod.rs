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
use doc::helper::{check_subtask_ids, forest, validate_forest};
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
    let tasks = if format.nests() {
        raw.iter()
            .map(RawTask::to_task)
            .collect::<Result<Vec<_>, Error>>()?
    } else {
        check(Limit::Tasks, raw.len()).map_err(Error::Validation)?;
        forest(
            raw.iter()
                .map(RawTask::to_flat_task)
                .collect::<Result<Vec<_>, Error>>()?,
        )?
    };
    validate_forest(&tasks).map_err(Error::Validation)?;
    version::check_document_versions(&tasks)?;
    Ok(tasks)
}

/// The version this document declares, or `None` when it declares none.
///
/// A syntax-level probe: it reports what the document claims, matching no
/// declaration against what this library implements.
pub fn detect_version(format: Format, input: &str) -> Result<Option<String>, Error> {
    detect::detect_version(format, input)
}

/// Exports tasks as a version-0 OPS document in `format`.
pub fn write(format: Format, tasks: &[Task]) -> Result<String, Error> {
    version::check_document_versions(tasks)?;
    validate_forest(tasks).map_err(Error::Validation)?;
    if !format.nests() {
        // A flat document references tasks by id, so every subtask needs one.
        check_subtask_ids(tasks).map_err(Error::Validation)?;
    }
    let doc = match format {
        Format::Json => Json.write(tasks),
        Format::Yaml => Yaml.write(tasks),
        Format::Jsonl => Jsonl.write(tasks),
        Format::Csv => Csv.write(tasks),
    }?;
    // A document we would refuse to read back is not worth writing.
    check(Limit::OutputBytes, doc.len()).map_err(Error::Validation)?;
    Ok(doc)
}
