//! The OPS data model, per the
//! [Open Productivity Standard spec](https://github.com/aricma/open-productivity-spec/blob/main/specs/v0/model.md).

pub(crate) mod fields;
/// The flat record shape, for the formats that reference tasks by id.
#[cfg_attr(not(any(feature = "jsonl", feature = "csv")), allow(dead_code))]
pub(crate) mod flat_task;
pub(crate) mod raw_task;
pub mod status;
pub mod task;
pub mod value;
