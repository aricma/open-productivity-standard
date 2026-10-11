//! The formats this crate claims. Which formats a spec version defines is
//! the specification's business; this is what `ops-lib` implements, one cargo
//! feature each. A build without a feature has no variant to name and no code
//! to read that syntax, which is what keeps a parser out of a build that does
//! not need it.

// The crate is a reader and a writer; with no format it could do neither, and
// the dispatch below would have nothing to dispatch to.
#[cfg(not(any(feature = "json", feature = "yaml", feature = "jsonl", feature = "csv")))]
compile_error!("enable at least one format: `json`, `yaml`, `jsonl`, or `csv`");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    #[cfg(feature = "json")]
    Json,
    #[cfg(feature = "yaml")]
    Yaml,
    #[cfg(feature = "jsonl")]
    Jsonl,
    #[cfg(feature = "csv")]
    Csv,
}

impl Format {
    /// Every format this build claims, so a caller can walk them.
    pub const ALL: &[Format] = &[
        #[cfg(feature = "json")]
        Format::Json,
        #[cfg(feature = "yaml")]
        Format::Yaml,
        #[cfg(feature = "jsonl")]
        Format::Jsonl,
        #[cfg(feature = "csv")]
        Format::Csv,
    ];

    pub fn name(self) -> &'static str {
        match self {
            #[cfg(feature = "json")]
            Format::Json => "json",
            #[cfg(feature = "yaml")]
            Format::Yaml => "yaml",
            #[cfg(feature = "jsonl")]
            Format::Jsonl => "jsonl",
            #[cfg(feature = "csv")]
            Format::Csv => "csv",
        }
    }

    /// Whether this format carries the tree as nested task objects rather
    /// than as child-id references. The other family is the flat one, whose
    /// documents reference tasks by id.
    pub const fn nests(self) -> bool {
        match self {
            #[cfg(feature = "json")]
            Format::Json => true,
            #[cfg(feature = "yaml")]
            Format::Yaml => true,
            #[cfg(feature = "jsonl")]
            Format::Jsonl => false,
            #[cfg(feature = "csv")]
            Format::Csv => false,
        }
    }

    /// The name this format goes by in diagnostics.
    pub const fn display_name(self) -> &'static str {
        match self {
            #[cfg(feature = "json")]
            Format::Json => "JSON",
            #[cfg(feature = "yaml")]
            Format::Yaml => "YAML",
            #[cfg(feature = "jsonl")]
            Format::Jsonl => "JSONL",
            #[cfg(feature = "csv")]
            Format::Csv => "CSV",
        }
    }

    pub fn from_name(name: &str) -> Option<Format> {
        Some(match name {
            #[cfg(feature = "json")]
            "json" => Format::Json,
            #[cfg(feature = "yaml")]
            "yaml" | "yml" => Format::Yaml,
            #[cfg(feature = "jsonl")]
            "jsonl" | "ndjson" => Format::Jsonl,
            #[cfg(feature = "csv")]
            "csv" => Format::Csv,
            _ => return None,
        })
    }
}

#[cfg(all(feature = "json", feature = "yaml", feature = "jsonl", feature = "csv"))]
#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn names_round_trip() {
        for format in Format::ALL {
            assert_eq!(Format::from_name(format.name()), Some(*format));
        }
        #[cfg(feature = "yaml")]
        assert_eq!(Format::from_name("yml"), Some(Format::Yaml));
        assert_eq!(Format::from_name("toml"), None);
    }

    /// Markdown is a format the specification defines, but not one this
    /// crate claims.
    #[test]
    fn markdown_is_not_a_format_this_crate_claims() {
        assert_eq!(Format::from_name("markdown"), None);
        assert_eq!(Format::from_name("md"), None);
    }

    #[test]
    fn the_families_are_nested_and_flat() {
        for format in Format::ALL {
            assert!(!format.display_name().is_empty());
            assert_eq!(Format::from_name(format.name()), Some(*format));
            // The tree formats list their children; the flat ones reference them.
            assert_eq!(format.nests(), matches!(format.name(), "json" | "yaml"));
        }
    }

    /// The formats this crate declares have to match `Format` exactly. The corpus
    /// generator reads `ops/formats.txt` instead of importing this crate — a tool
    /// about the standard's corpus should not depend on one implementation of it —
    /// so this file is the one place the claim is written twice, and a format in
    /// one and not the other would quietly stop being covered by the corpus.
    ///
    /// The claim is the whole set, so the check is for a build that has it.
    #[test]
    fn the_declared_formats_match_the_format_enum() {
        let declared: Vec<Vec<String>> = include_str!("../../formats.txt")
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.split_whitespace().map(str::to_string).collect())
            .filter(|words: &Vec<String>| words[0] != "crate")
            .collect();
        assert_eq!(
            declared.len(),
            Format::ALL.len(),
            "ops/formats.txt declares {} formats, this crate claims {}",
            declared.len(),
            Format::ALL.len()
        );

        for format in Format::ALL {
            let name = format.name();
            let variant = format!("{format:?}");
            let line = declared
                .iter()
                .find(|words| words[0] == name)
                .unwrap_or_else(|| panic!("ops/formats.txt does not declare `{name}`"));
            assert_eq!(
                line.get(1).map(String::as_str),
                Some(variant.as_str()),
                "ops/formats.txt pairs `{name}` with the wrong variant"
            );
            // An extension the file names has to be one this crate reads as the
            // same format.
            if let Some(extension) = line.get(2) {
                assert_eq!(
                    Format::from_name(extension),
                    Some(*format),
                    "ops/formats.txt writes `{name}` as `.{extension}`, which this crate does not read as {variant}"
                );
            }
        }
    }
}
