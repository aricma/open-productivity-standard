//! What a case is: one corpus directory, and what it asserts.

use crate::formats::Implementation;

/// The kinds of case a format directory holds. Every kind but `Transform` lives
/// inside one format; `Transform` crosses formats and sits at the top.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Group {
    Valid,
    Invalid,
    Roundtrip,
    Transform,
}

impl Group {
    /// Every kind, in the order its module is listed.
    pub(crate) const ALL: [Group; 4] = [
        Group::Valid,
        Group::Invalid,
        Group::Roundtrip,
        Group::Transform,
    ];

    /// The directory a case of this kind lives in.
    pub(crate) fn dir(self) -> &'static str {
        match self {
            Group::Valid => "valid",
            Group::Invalid => "invalid",
            Group::Roundtrip => "roundtrip",
            Group::Transform => "transform",
        }
    }

    /// Whether the kind sits at the corpus top instead of inside a format.
    pub(crate) fn crosses_formats(self) -> bool {
        matches!(self, Group::Transform)
    }
}

/// One corpus directory, read.
pub(crate) struct Case {
    pub(crate) version: String,
    /// The enclosing format directory; `None` for a `transform/` case.
    pub(crate) format: Option<String>,
    pub(crate) group: Group,
    pub(crate) name: String,
    pub(crate) claim: String,
    pub(crate) given_ext: String,
    /// Relative to the repository root, for `include_str!`.
    pub(crate) given: String,
    /// What this case writes, if it writes at all: the format its result must
    /// be in.
    pub(crate) target_ext: Option<String>,
    /// `(extension, path relative to the repository root)` of the document the
    /// result must equal. Absent from a conversion that must fail.
    pub(crate) expectation: Option<(String, String)>,
}

impl Case {
    /// Where the case lives, relative to the repository root.
    pub(crate) fn corpus_path(&self) -> String {
        match &self.format {
            Some(format) => format!(
                "specs/{}/tests/{format}/{}/{}",
                self.version,
                self.group.dir(),
                self.name
            ),
            None => format!(
                "specs/{}/tests/{}/{}",
                self.version,
                self.group.dir(),
                self.name
            ),
        }
    }

    /// Whether this case asserts a conversion that cannot be done: the model is
    /// valid, and the format it is asked to write simply cannot carry it.
    pub(crate) fn must_fail(&self) -> bool {
        self.group == Group::Transform && self.expectation.is_none()
    }

    /// The source paths this case reads from and writes to, as
    /// `ops_lib::v0::Format::…`. The write is absent when the case only reads.
    pub(crate) fn format_paths(
        &self,
        implementation: &Implementation,
    ) -> (Option<String>, Option<String>) {
        let reads = implementation.variant_path(&self.version, &self.given_ext);
        let writes = self
            .target_ext
            .as_deref()
            .and_then(|target| implementation.variant_path(&self.version, target));
        (reads, writes)
    }

    /// Every format name or extension this case exercises, the one it reads
    /// first. A case runs only when all of them are claimed.
    pub(crate) fn format_names(&self) -> Vec<&str> {
        let mut names = vec![self.given_ext.as_str()];
        if let Some(target) = &self.target_ext {
            names.push(target);
        }
        names
    }
}

/// The test function name for a case directory: snake_case, so the directory
/// name reads as the statement it is.
pub(crate) fn test_name(case_dir_name: &str) -> String {
    let mut name = String::new();
    let mut last_was_separator = false;
    for character in case_dir_name.to_lowercase().chars() {
        if character.is_alphanumeric() {
            name.push(character);
            last_was_separator = false;
        } else if !last_was_separator {
            name.push('_');
            last_was_separator = true;
        }
    }
    name.trim_end_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;
    use std::path::Path;

    /// What the repository's implementation declares.
    fn implementation() -> Implementation {
        Implementation::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ops/formats.txt"))
    }

    fn case(
        group: Group,
        format: Option<&str>,
        given: &str,
        target: Option<&str>,
        expectation: Option<&str>,
    ) -> Case {
        Case {
            version: "v0".to_string(),
            format: format.map(str::to_string),
            group,
            name: "case".to_string(),
            claim: "the claim".to_string(),
            given_ext: given.to_string(),
            given: format!("specs/v0/tests/place/given.{given}"),
            target_ext: target.map(str::to_string),
            expectation: expectation.map(|ext| {
                (
                    ext.to_string(),
                    format!("specs/v0/tests/place/expectation.{ext}"),
                )
            }),
        }
    }

    /// The paths a case needs, which is what decides whether it can run at all.
    #[test]
    fn a_case_names_the_formats_it_reads_and_writes() {
        let reads_only = case(Group::Valid, Some("json"), "json", None, None);
        assert_eq!(
            reads_only.format_paths(&implementation()),
            (Some("ops_lib::v0::Format::Json".to_string()), None)
        );

        let roundtrip = case(
            Group::Roundtrip,
            Some("json"),
            "json",
            Some("json"),
            Some("json"),
        );
        assert_eq!(
            roundtrip.format_paths(&implementation()),
            (
                Some("ops_lib::v0::Format::Json".to_string()),
                Some("ops_lib::v0::Format::Json".to_string())
            )
        );

        // A transform writes the target, not the format it read.
        let transform = case(Group::Transform, None, "json", Some("yaml"), Some("yaml"));
        assert_eq!(
            transform.format_paths(&implementation()),
            (
                Some("ops_lib::v0::Format::Json".to_string()),
                Some("ops_lib::v0::Format::Yaml".to_string())
            )
        );
    }

    /// A conversion with nothing to compare against asserts that it fails, and
    /// that is the only kind of case filed by a model that is itself valid.
    #[test]
    fn a_conversion_without_an_expectation_must_fail() {
        let impossible = case(Group::Transform, None, "json", Some("jsonl"), None);
        assert!(impossible.must_fail());
        assert_eq!(impossible.format_names(), vec!["json", "jsonl"]);

        let possible = case(Group::Transform, None, "json", Some("yaml"), Some("yaml"));
        assert!(!possible.must_fail());

        // Only a conversion may assert a failure; an `invalid` case is a
        // document the reader must reject.
        let invalid = case(Group::Invalid, Some("jsonl"), "jsonl", None, None);
        assert!(!invalid.must_fail());
    }

    #[test]
    fn a_corpus_path_says_where_the_case_lives() {
        assert_eq!(
            case(Group::Valid, Some("json"), "json", None, None).corpus_path(),
            "specs/v0/tests/json/valid/case"
        );
        assert_eq!(
            case(Group::Transform, None, "json", Some("yaml"), Some("yaml")).corpus_path(),
            "specs/v0/tests/transform/case"
        );
    }

    #[test]
    fn test_names_are_snake_case() {
        assert_eq!(
            test_name("A title is one non-empty line."),
            "a_title_is_one_non_empty_line"
        );
        assert_eq!(test_name("json-to-yaml"), "json_to_yaml");
        assert_eq!(test_name("trailing"), "trailing");
    }
}
