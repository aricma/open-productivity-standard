//! The formats this crate claims. Which formats a spec version defines is
//! the specification's business; this is what `ops-lib` implements.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Json,
    Yaml,
    Jsonl,
    Csv,
}

impl Format {
    /// Every format this crate claims, in no particular order, so a caller can
    /// walk them.
    pub const ALL: [Format; 4] = [Format::Json, Format::Yaml, Format::Jsonl, Format::Csv];

    pub fn name(&self) -> &'static str {
        match self {
            Format::Json => "json",
            Format::Yaml => "yaml",
            Format::Jsonl => "jsonl",
            Format::Csv => "csv",
        }
    }

    /// Whether this format carries the tree as nested task objects rather
    /// than as child-id references. The other family is the flat one, whose
    /// documents reference tasks by id.
    pub const fn nests(self) -> bool {
        matches!(self, Format::Json | Format::Yaml)
    }

    /// The name this format goes by in diagnostics.
    pub const fn display_name(self) -> &'static str {
        match self {
            Format::Json => "JSON",
            Format::Yaml => "YAML",
            Format::Jsonl => "JSONL",
            Format::Csv => "CSV",
        }
    }

    pub fn from_name(name: &str) -> Option<Format> {
        Some(match name {
            "json" => Format::Json,
            "yaml" | "yml" => Format::Yaml,
            "jsonl" | "ndjson" => Format::Jsonl,
            "csv" => Format::Csv,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn names_round_trip() {
        for format in [Format::Json, Format::Yaml, Format::Jsonl, Format::Csv] {
            assert_eq!(Format::from_name(format.name()), Some(format));
        }
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
        assert!(Format::Json.nests());
        assert!(Format::Yaml.nests());
        assert!(!Format::Jsonl.nests());
        assert!(!Format::Csv.nests());
        for format in Format::ALL {
            assert!(!format.display_name().is_empty());
            assert_eq!(Format::from_name(format.name()), Some(format));
        }
    }

    /// The formats this crate declares have to match `Format` exactly. The corpus
    /// generator reads `ops/formats.txt` instead of importing this crate — a tool
    /// about the standard's corpus should not depend on one implementation of it —
    /// so this file is the one place the claim is written twice, and a format in
    /// one and not the other would quietly stop being covered by the corpus.
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
                    Some(format),
                    "ops/formats.txt writes `{name}` as `.{extension}`, which this crate does not read as {variant}"
                );
            }
        }
    }
}
