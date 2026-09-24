#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Json,
    Yaml,
    Jsonl,
}

impl Format {
    pub fn name(&self) -> &'static str {
        match self {
            Format::Json => "json",
            Format::Yaml => "yaml",
            Format::Jsonl => "jsonl",
        }
    }

    pub fn from_name(name: &str) -> Option<Format> {
        Some(match name {
            "json" => Format::Json,
            "yaml" | "yml" => Format::Yaml,
            "jsonl" | "ndjson" => Format::Jsonl,
            _ => return None,
        })
    }
}
