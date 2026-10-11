//! What a case is: one corpus directory, and what it asserts.

use crate::formats;

/// The kinds of case a format directory holds. Every kind but `Transform` lives
/// inside one format; `Transform` crosses formats and sits at the top.
///
/// `Invalid` runs both ways: a `given` in the enclosing format is rejected by
/// its reader, and a `given` in another format is a valid model that only the
/// enclosing format's writer refuses.
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

    /// Whether a case of this kind carries an expectation file.
    pub(crate) fn requires_expectation(self) -> bool {
        matches!(self, Group::Roundtrip | Group::Transform)
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
    /// `(extension, path relative to the repository root)`.
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

    /// Whether this case hands the enclosing format a model it cannot carry, so
    /// the failure is on write rather than on read. Inside a format directory,
    /// an `invalid` case whose `given` is in another format is that shape.
    pub(crate) fn rejects_the_writer(&self) -> bool {
        let (Some(format), Group::Invalid) = (&self.format, self.group) else {
            return false;
        };
        !formats::same(&self.version, format, &self.given_ext)
    }

    /// The source paths this case reads from and writes to, as
    /// `ops_lib::v0::Format::…`. The write is absent when the case only reads.
    pub(crate) fn format_paths(&self) -> (Option<String>, Option<String>) {
        let reads = formats::variant_path(&self.version, &self.given_ext);
        let writes = if self.rejects_the_writer() {
            self.format
                .as_deref()
                .and_then(|format| formats::variant_path(&self.version, format))
        } else {
            match self.group {
                Group::Roundtrip => reads.clone(),
                Group::Transform => self
                    .expectation
                    .as_ref()
                    .and_then(|(ext, _)| formats::variant_path(&self.version, ext)),
                Group::Valid | Group::Invalid => None,
            }
        };
        (reads, writes)
    }

    /// Every format name or extension this case exercises.
    pub(crate) fn format_names(&self) -> Vec<&str> {
        let mut names = vec![self.given_ext.as_str()];
        if let Some((ext, _)) = &self.expectation {
            names.push(ext);
        }
        if self.rejects_the_writer() {
            names.push(
                self.format
                    .as_deref()
                    .expect("a writer-reject case is inside a format directory"),
            );
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

    fn case(group: Group, format: Option<&str>, given: &str, expectation: Option<&str>) -> Case {
        Case {
            version: "v0".to_string(),
            format: format.map(str::to_string),
            group,
            name: "case".to_string(),
            claim: "the claim".to_string(),
            given_ext: given.to_string(),
            given: format!("specs/v0/tests/place/given.{given}"),
            expectation: expectation.map(|ext| {
                (
                    ext.to_string(),
                    format!("specs/v0/tests/place/expectation.{ext}"),
                )
            }),
        }
    }

    /// Which direction an `invalid` case runs: the enclosing format's reader, or
    /// — when the given is in another format — its writer.
    #[test]
    fn invalid_runs_the_way_the_given_says() {
        assert!(!case(Group::Invalid, Some("json"), "json", None).rejects_the_writer());
        assert!(case(Group::Invalid, Some("markdown"), "json", None).rejects_the_writer());
        assert!(!case(Group::Valid, Some("markdown"), "md", None).rejects_the_writer());
    }

    /// The paths a case needs, which is what decides whether it can run at all.
    #[test]
    fn a_case_names_the_formats_it_reads_and_writes() {
        let reads_only = case(Group::Valid, Some("json"), "json", None);
        assert_eq!(
            reads_only.format_paths(),
            (Some("ops_lib::v0::Format::Json".to_string()), None)
        );

        let roundtrip = case(Group::Roundtrip, Some("json"), "json", Some("json"));
        assert_eq!(
            roundtrip.format_paths(),
            (
                Some("ops_lib::v0::Format::Json".to_string()),
                Some("ops_lib::v0::Format::Json".to_string())
            )
        );

        // A transform writes the expectation's format, not the given's.
        let transform = case(Group::Transform, None, "json", Some("yaml"));
        assert_eq!(
            transform.format_paths(),
            (
                Some("ops_lib::v0::Format::Json".to_string()),
                Some("ops_lib::v0::Format::Yaml".to_string())
            )
        );

        // A writer-reject case writes the format it is filed under.
        let refused = case(Group::Invalid, Some("markdown"), "json", None);
        assert_eq!(
            refused.format_paths(),
            (
                Some("ops_lib::v0::Format::Json".to_string()),
                None // Markdown is not claimed, so there is no path for it.
            )
        );
        assert_eq!(refused.format_names(), vec!["json", "markdown"]);
    }

    #[test]
    fn a_corpus_path_says_where_the_case_lives() {
        assert_eq!(
            case(Group::Valid, Some("json"), "json", None).corpus_path(),
            "specs/v0/tests/json/valid/case"
        );
        assert_eq!(
            case(Group::Transform, None, "json", Some("yaml")).corpus_path(),
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
