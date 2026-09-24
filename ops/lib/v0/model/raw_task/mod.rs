//! Lenient document shape: fields are read as `serde_json::Value` so a
//! missing or mistyped field is a coded rejection instead of a raw serde
//! message. Graph semantics stay in `helper`.
//!
//! - the `model` submodule is the wire shape, its conversions, and its tests,
//! - `validators` holds the per-field checks,
//! - `helpers` holds the small readers both use.

mod helpers;
mod model;
mod validators;

pub(crate) use model::RawTask;
