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
        for format in [Format::Json, Format::Yaml, Format::Jsonl, Format::Csv] {
            assert!(!format.display_name().is_empty());
        }
    }
}
