//! What the per-format targets share: the document every format has to carry,
//! and the claim each build makes about the formats the standard documents.
//!
//! Each target uses part of this, so the rest is dead in that binary.

#![allow(dead_code)]

use ops_lib::v0::{Format, Metadata, MetadataValue, Status, Task};

/// A document every format carries: one root with every field the model has,
/// and one child, which a flat format reaches by id. The metadata holds a
/// string that parses as JSON, which CSV has to route to its `metadata` column
/// rather than to a bare cell.
pub fn document() -> Vec<Task> {
    let mut metadata = Metadata::new();
    metadata.insert(
        "priority".to_string(),
        MetadataValue::String("high".to_string()),
    );
    metadata.insert(
        "tags".to_string(),
        MetadataValue::List(vec![
            MetadataValue::String("infra".to_string()),
            MetadataValue::String("auth".to_string()),
        ]),
    );
    metadata.insert("count".to_string(), MetadataValue::Number(3_i64.into()));
    metadata.insert("note".to_string(), MetadataValue::String("3".to_string()));

    vec![Task {
        title: "Root".to_string(),
        status: Status::Open,
        id: Some("root".to_string()),
        version: Some("0".to_string()),
        notes: Some("first line\nsecond line".to_string()),
        metadata: Some(metadata),
        subtasks: vec![Task {
            title: "Child".to_string(),
            status: Status::Done,
            id: Some("child".to_string()),
            ..Default::default()
        }],
    }]
}

/// Writes the document in `format`, reads it back, and requires the same model.
pub fn round_trip(format: Format) {
    let written = ops_lib::v0::write(format, &document())
        .unwrap_or_else(|e| panic!("{}: write: {e}", format.name()));
    let read = ops_lib::v0::read(format, &written)
        .unwrap_or_else(|e| panic!("{}: read: {e}\n{written}", format.name()));
    assert_eq!(
        read,
        document(),
        "{} round-trip changed the model\n{written}",
        format.name()
    );
}

/// One format as `ops/formats.txt` declares it: the name the standard uses, and
/// the file extension when that is a different spelling.
pub struct Declared {
    pub name: String,
    pub extension: Option<String>,
}

pub fn declared_formats() -> Vec<Declared> {
    include_str!("../../../../ops/formats.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            line.split_whitespace()
                .map(str::to_string)
                .collect::<Vec<String>>()
        })
        .filter(|words: &Vec<String>| words[0] != "crate")
        .map(|words| Declared {
            name: words[0].clone(),
            extension: words.get(2).cloned(),
        })
        .collect()
}

/// Whether this build enables the feature that claims `name`.
pub fn claims(name: &str) -> bool {
    match name {
        "json" => cfg!(feature = "json"),
        "yaml" => cfg!(feature = "yaml"),
        "jsonl" => cfg!(feature = "jsonl"),
        "csv" => cfg!(feature = "csv"),
        other => panic!("ops/formats.txt declares `{other}`, which this test does not know"),
    }
}

/// Every format the standard documents is a format here exactly when its
/// feature is on — there is no variant to pass to `read` or `write` when it is
/// off — and a syntax OPS does not define never is.
pub fn assert_formats_are_claimed_exactly_when_their_feature_is_on() {
    for declared in declared_formats() {
        let name = declared.name.as_str();
        if claims(name) {
            let format = Format::from_name(name)
                .unwrap_or_else(|| panic!("`{name}` is enabled but is not a format"));
            assert_eq!(format.name(), name);
            assert!(Format::ALL.contains(&format));
        } else {
            assert_eq!(
                Format::from_name(name),
                None,
                "`{name}` resolved without its feature"
            );
            if let Some(extension) = &declared.extension {
                assert_eq!(
                    Format::from_name(extension),
                    None,
                    "`.{extension}` resolved without its feature"
                );
            }
            assert!(
                Format::ALL.iter().all(|format| format.name() != name),
                "`{name}` is in Format::ALL without its feature"
            );
        }
    }

    // Markdown is a format the specification defines and no build claims.
    for name in ["markdown", "md"] {
        assert_eq!(
            Format::from_name(name),
            None,
            "`{name}` is not an OPS format"
        );
        assert!(Format::ALL.iter().all(|format| format.name() != name));
    }
}
