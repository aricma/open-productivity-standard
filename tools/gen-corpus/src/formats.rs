//! What the implementation declares it claims.
//!
//! The tool does not import the implementation: it reads `ops/formats.txt`,
//! which the implementation writes down and its own tests check against
//! `Format`. So this crate knows a *declaration*, not a crate, and pointing it
//! at another implementation means pointing it at another declaration.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// A format this implementation does not claim.
pub(crate) struct Unsupported(pub(crate) String);

/// An implementation's declared formats.
pub(crate) struct Implementation {
    /// The crate its modules live in, as a Rust path: `ops_lib`.
    crate_path: String,
    /// What each name the corpus uses resolves to, so `markdown` and `md` are
    /// recognisable as the same format.
    declared: BTreeMap<String, Declared>,
}

/// One declared format: the variant it is read and written with, and the file
/// extension if that differs from the name.
struct Declared {
    variant: String,
    /// Every name this format answers to.
    names: Vec<String>,
}

impl Implementation {
    /// Read what the implementation at `path` declares.
    pub(crate) fn read(path: &Path) -> Implementation {
        let text =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let mut crate_path = None;
        let mut declared: BTreeMap<String, Declared> = BTreeMap::new();

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let words: Vec<&str> = line.split_whitespace().collect();
            match words.as_slice() {
                ["crate", path] => crate_path = Some((*path).to_string()),
                [name, variant] => {
                    declared.insert(
                        (*name).to_string(),
                        Declared {
                            variant: (*variant).to_string(),
                            names: vec![(*name).to_string()],
                        },
                    );
                }
                // The extension answers to the same format, so a case may use
                // it without looking like a different one.
                [name, variant, extension] => {
                    declared.insert(
                        (*name).to_string(),
                        Declared {
                            variant: (*variant).to_string(),
                            names: vec![(*name).to_string(), (*extension).to_string()],
                        },
                    );
                }
                _ => panic!("{}: `{line}` is not a format declaration", path.display()),
            }
        }

        let crate_path =
            crate_path.unwrap_or_else(|| panic!("{}: no `crate` line", path.display()));
        assert!(
            !declared.is_empty(),
            "{}: declares no formats",
            path.display()
        );
        Implementation {
            crate_path,
            declared,
        }
    }

    /// Whether this implementation claims a format at all.
    pub(crate) fn claims(&self, format: &str) -> bool {
        self.variant_name(format).is_some()
    }

    /// Whether two names denote the same format.
    ///
    /// A name the implementation does not claim cannot be resolved to a format,
    /// so two unclaimed names are taken as the same one: it is not a format this
    /// implementation can tell apart, and its cases are skipped either way. The
    /// declaration's extension column is what resolves the pair once the format
    /// *is* claimed.
    pub(crate) fn same(&self, one: &str, other: &str) -> bool {
        match (self.variant_name(one), self.variant_name(other)) {
            (Some(one), Some(other)) => one == other,
            (None, None) => true,
            _ => false,
        }
    }

    /// The source path a case needs for one format: `ops_lib::v0::Format::Json`.
    pub(crate) fn variant_path(&self, version: &str, format: &str) -> Option<String> {
        let crate_path = &self.crate_path;
        self.variant_name(format)
            .map(|variant| format!("{crate_path}::{version}::Format::{variant}"))
    }

    /// The variant a name resolves to, through the format it denotes.
    fn variant_name(&self, format: &str) -> Option<&str> {
        self.declared
            .values()
            .find(|declared| declared.names.iter().any(|name| name == format))
            .map(|declared| declared.variant.as_str())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    /// What the repository's own implementation declares.
    fn declared() -> Implementation {
        Implementation::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ops/formats.txt"))
    }

    #[test]
    fn a_claimed_format_has_a_path_and_an_unclaimed_one_does_not() {
        let implementation = declared();
        assert_eq!(
            implementation.variant_path("v0", "json"),
            Some("ops_lib::v0::Format::Json".to_string())
        );
        assert!(implementation.claims("csv"));
        assert!(!implementation.claims("markdown"));
        // A version this implementation has no module for.
        assert_eq!(
            implementation.variant_path("v9", "json").as_deref(),
            Some("ops_lib::v9::Format::Json")
        );
    }

    #[test]
    fn a_format_can_be_named_two_ways() {
        let implementation = declared();
        assert!(implementation.same("yaml", "yaml"));
        assert!(!implementation.same("json", "yaml"));
        // A format this implementation does not claim resolves to nothing, so
        // any name for it is that unresolved format — including its extension,
        // which is why the declaration needs no line for it.
        assert!(implementation.same("markdown", "markdown"));
        assert!(implementation.same("markdown", "md"));
        // A claimed one and an unclaimed one are never the same format.
        assert!(!implementation.same("json", "markdown"));
    }

    /// An extension that differs from the format's name still denotes it.
    #[test]
    fn a_declared_extension_stands_for_its_format() {
        let path = std::env::temp_dir().join("gen-corpus-formats.txt");
        fs::write(&path, "crate ops_lib\nmarkdown Markdown md\n").unwrap();
        let implementation = Implementation::read(&path);
        assert!(implementation.same("markdown", "md"));
        assert_eq!(
            implementation.variant_path("v0", "md"),
            Some("ops_lib::v0::Format::Markdown".to_string())
        );
    }

    #[test]
    #[should_panic(expected = "is not a format declaration")]
    fn a_malformed_declaration_is_not_ignored() {
        let path = std::env::temp_dir().join("gen-corpus-formats-bad.txt");
        fs::write(&path, "crate ops_lib\njson Json extra more\n").unwrap();
        Implementation::read(&path);
    }
}
