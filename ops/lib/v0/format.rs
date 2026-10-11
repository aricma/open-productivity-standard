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
}
