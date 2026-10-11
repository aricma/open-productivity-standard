//! The OPS versions this library implements.
//!
//! Which versions an implementation accepts is its own decision: the standard
//! only asks that a document's declaration be findable, and the allowed set
//! depends on what the specifications have released and what a tool supports.
//! So this list is ops-lib policy, not part of OPS.
//!
//! Version-independent: the list and the membership rule are the same for
//! every spec version.

/// The OPS versions this library implements. Add one when its module lands.
pub(crate) const SUPPORTED_VERSIONS: &[&str] = &["0"];

/// The first used version this library does not implement, if any.
pub(crate) fn first_unsupported<'a>(used: impl IntoIterator<Item = &'a str>) -> Option<String> {
    used.into_iter()
        .find(|v| !SUPPORTED_VERSIONS.contains(v))
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_only_unsupported_versions() {
        assert_eq!(first_unsupported(["0", "0"]), None);
        assert_eq!(first_unsupported(["0", "1"]).as_deref(), Some("1"));
        assert_eq!(first_unsupported(["9", "1"]).as_deref(), Some("9"));
        assert_eq!(first_unsupported(std::iter::once("0")), None);
        assert_eq!(first_unsupported(std::iter::empty::<&str>()), None);
    }
}
