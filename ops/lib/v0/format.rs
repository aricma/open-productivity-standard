#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Json,
    Yaml,
    Jsonl,
    Csv,
    Md,
}

impl Format {
    pub fn name(&self) -> &'static str {
        match self {
            Format::Json => "json",
            Format::Yaml => "yaml",
            Format::Jsonl => "jsonl",
            Format::Csv => "csv",
            Format::Md => "md",
        }
    }

    pub fn from_name(name: &str) -> Option<Format> {
        Some(match name {
            "json" => Format::Json,
            "yaml" | "yml" => Format::Yaml,
            "jsonl" | "ndjson" => Format::Jsonl,
            "csv" => Format::Csv,
            "md" | "markdown" => Format::Md,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for format in [
            Format::Json,
            Format::Yaml,
            Format::Jsonl,
            Format::Csv,
            Format::Md,
        ] {
            assert_eq!(Format::from_name(format.name()), Some(format));
        }
        assert_eq!(Format::from_name("markdown"), Some(Format::Md));
        assert_eq!(Format::from_name("yml"), Some(Format::Yaml));
        assert_eq!(Format::from_name("toml"), None);
    }
}
