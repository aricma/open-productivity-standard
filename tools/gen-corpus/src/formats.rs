//! How a corpus format name maps to the reference implementation's formats.
//!
//! The corpus names formats the way the standard documents them — `json`,
//! `markdown` — and the implementation names them its own way, so this is the
//! one place that knows both vocabularies. A format the implementation does
//! not claim is not a failure: its cases become `#[ignore]`d tests.

use ops_lib::v0::Format;

/// A format this implementation does not claim.
pub(crate) struct Unsupported(pub(crate) String);

/// The variant the implementation uses, as source: `Json`.
pub(crate) fn variant_name(version: &str, format: &str) -> Option<String> {
    match version {
        "v0" => Format::from_name(format).map(|format| format!("{format:?}")),
        _ => None,
    }
}

/// The full source path for a variant: `ops_lib::v0::Format::Json`.
pub(crate) fn variant_path(version: &str, format: &str) -> Option<String> {
    variant_name(version, format).map(|name| format!("ops_lib::{version}::Format::{name}"))
}

/// Whether two names denote the same format. Names of formats the
/// implementation does not claim compare as names, since it has no variant for
/// them.
pub(crate) fn same(version: &str, one: &str, other: &str) -> bool {
    variant_name(version, one) == variant_name(version, other)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_claimed_format_has_a_variant_and_an_unclaimed_one_does_not() {
        assert_eq!(variant_name("v0", "json"), Some("Json".to_string()));
        assert_eq!(
            variant_path("v0", "csv"),
            Some("ops_lib::v0::Format::Csv".to_string())
        );
        // A name this version does not claim, and a version this binary has no
        // module for.
        assert_eq!(variant_name("v0", "markdown"), None);
        assert_eq!(variant_name("v9", "json"), None);
    }

    #[test]
    fn the_same_format_can_be_named_two_ways() {
        assert!(same("v0", "yaml", "yml"));
        assert!(same("v0", "markdown", "md"));
        assert!(!same("v0", "json", "yaml"));
    }
}
