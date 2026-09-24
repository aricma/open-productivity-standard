//! The official OPS release list, and the rule that a document's
//! declared `version` must be one of them.
//!
//! Version-independent: the list and the membership rule are the same
//! for every spec version. Each version walks its own model and maps the
//! result onto its own error type.

/// Released OPS versions. Add a version once it officially ships.
pub(crate) const RELEASED_VERSIONS: &[&str] = &["0"];

/// The first used version that is not an official release, if any.
pub(crate) fn first_unreleased<'a>(used: impl IntoIterator<Item = &'a str>) -> Option<String> {
    used.into_iter()
        .find(|v| !RELEASED_VERSIONS.contains(v))
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_only_unreleased_versions() {
        assert_eq!(first_unreleased(["0", "0"]), None);
        assert_eq!(first_unreleased(["0", "1"]).as_deref(), Some("1"));
        assert_eq!(first_unreleased(["9", "1"]).as_deref(), Some("9"));
        assert_eq!(first_unreleased(std::iter::once("0")), None);
        assert_eq!(first_unreleased(std::iter::empty::<&str>()), None);
    }
}
