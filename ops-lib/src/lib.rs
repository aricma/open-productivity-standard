#![deny(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable
)]

//! Reader and writer for the Open Productivity Standard (OPS).
//!
//! Spec: <https://github.com/aricma/open-productivity-spec>
//!
//! Implements the OPS task model (one `task` type, nested arbitrarily
//! deep) and these serializations:
//!
//! - tree-preserving: JSON, YAML
//! - flat/streamable: JSONL
//!
//! CSV and Markdown are specced but not implemented yet.

mod config;
mod error;
mod format;
mod helpers;

pub mod doc;
pub mod model;

use doc::OpsDoc;

pub use doc::json::Json;
pub use doc::jsonl::Jsonl;
pub use doc::yaml::Yaml;
pub use error::{Error, ValidationError};
pub use format::Format;
pub use model::status::Status;
pub use model::task::Task;

pub fn read(format: Format, input: &str) -> Result<Vec<Task>, Error> {
    let tasks = match format {
        Format::Json => Json.parse(input)?,
        Format::Yaml => Yaml.parse(input)?,
        Format::Jsonl => Jsonl.parse(input)?,
    };
    helpers::validate_used_ops_version_against_official_releases(&tasks)?;
    Ok(tasks)
}

pub fn write(format: Format, tasks: &[Task]) -> Result<String, Error> {
    helpers::validate_used_ops_version_against_official_releases(tasks)?;
    match format {
        Format::Json => Json.write(tasks),
        Format::Yaml => Yaml.write(tasks),
        Format::Jsonl => Jsonl.write(tasks),
    }
}
