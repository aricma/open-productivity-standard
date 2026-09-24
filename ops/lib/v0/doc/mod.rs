//! The serialization layer: the [`OpsDoc`] trait in its own module, one
//! module per format (`json`, `yaml`, `jsonl`), and shared helpers.

pub(crate) mod helper;
pub(crate) mod json;
pub(crate) mod jsonl;
pub(crate) mod ops_doc;
pub(crate) mod yaml;

pub(crate) use ops_doc::OpsDoc;
