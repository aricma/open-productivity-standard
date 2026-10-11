//! The serialization layer: the [`OpsDoc`] trait in its own module, one
//! module per format the build enables (`json`, `yaml`, `jsonl`, `csv`), and
//! shared helpers.

#[cfg(feature = "csv")]
pub(crate) mod csv;
pub(crate) mod helper;
#[cfg(feature = "json")]
pub(crate) mod json;
#[cfg(feature = "jsonl")]
pub(crate) mod jsonl;
pub(crate) mod ops_doc;
#[cfg(feature = "yaml")]
pub(crate) mod yaml;

pub(crate) use ops_doc::OpsDoc;
