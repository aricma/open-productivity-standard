//! CSV serialization (flat/streamable forest): one task per row, the first
//! row naming the columns. Reserved columns map to the model fields; every
//! other column is metadata, optionally `metadata_`-prefixed and
//! type-suffixed (RFC-014).

use super::helper::records;
use crate::v0::doc::OpsDoc;
use crate::v0::error::{Error, ValidationError};
use crate::v0::model::raw_task::RawTask;
use crate::v0::model::task::Task;
use crate::v0::model::value::{Metadata, MetadataValue};
use serde_json::{Map, Value};
use std::collections::{BTreeSet, HashSet};

pub struct Csv;

const RESERVED: [&str; 7] = [
    "version", "id", "title", "status", "notes", "subtasks", "metadata",
];

#[derive(Clone, Copy)]
enum CellType {
    Json,
    String,
    Number,
    Bool,
}

/// One header cell: a reserved model field, or a metadata column.
enum Column {
    Field(&'static str),
    Metadata {
        key: String,
        typed: Option<CellType>,
    },
}

fn classify(header: &str) -> Column {
    if let Some(field) = RESERVED.into_iter().find(|field| *field == header) {
        return Column::Field(field);
    }
    let (rest, typed) = match header.strip_prefix("metadata_") {
        Some(rest) => (rest, type_suffix(rest)),
        None => (header, None),
    };
    let key = match typed {
        Some((_, key)) => key,
        None => rest,
    };
    // The derived key is checked by the model's metadata-key rule, not here: an
    // out-of-charset column is stored as-is and rejected by `validate_forest`.
    Column::Metadata {
        key: key.to_string(),
        typed: typed.map(|(ty, _)| ty),
    }
}

fn type_suffix(s: &str) -> Option<(CellType, &str)> {
    const SUFFIXES: [(&str, CellType); 4] = [
        ("_json", CellType::Json),
        ("_string", CellType::String),
        ("_number", CellType::Number),
        ("_bool", CellType::Bool),
    ];
    SUFFIXES
        .into_iter()
        .find_map(|(suffix, ty)| s.strip_suffix(suffix).map(|key| (ty, key)))
}

/// RFC 4180 records: `"` quoting with `""` escapes, embedded newlines, LF or
/// CRLF terminators. Blank lines are skipped.
pub(crate) fn read_records(input: &str) -> Result<Vec<Vec<String>>, Error> {
    let mut records: Vec<Vec<String>> = Vec::new();
    let mut record: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut started = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    quoted = false;
                }
            } else {
                field.push(c);
            }
            continue;
        }
        match c {
            '"' if field.is_empty() => {
                quoted = true;
                started = true;
            }
            ',' => {
                record.push(std::mem::take(&mut field));
                started = true;
            }
            '\n' => {
                if started || !field.is_empty() || !record.is_empty() {
                    record.push(std::mem::take(&mut field));
                    records.push(std::mem::take(&mut record));
                }
                started = false;
            }
            '\r' => {}
            _ => {
                field.push(c);
                started = true;
            }
        }
    }
    if quoted {
        return Err(Error::syntax("CSV", "unterminated quoted field"));
    }
    if started || !field.is_empty() || !record.is_empty() {
        record.push(field);
        records.push(record);
    }
    Ok(records)
}

fn cell(text: &str) -> String {
    if text.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text.to_string()
    }
}

fn write_record<S: AsRef<str>>(fields: &[S]) -> String {
    let mut line = fields
        .iter()
        .map(|f| cell(f.as_ref()))
        .collect::<Vec<_>>()
        .join(",");
    line.push('\n');
    line
}

/// A cell that parses as JSON is that value; anything else is a string.
fn inferred_cell(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.to_string()))
}

/// A typed cell, kept as a string when it cannot be read as that type.
fn typed_cell(text: &str, ty: CellType) -> Value {
    let Ok(parsed) = serde_json::from_str::<Value>(text) else {
        return Value::String(text.to_string());
    };
    let matches = match ty {
        CellType::Json => true,
        CellType::String => false,
        CellType::Number => parsed.is_number(),
        CellType::Bool => parsed.is_boolean(),
    };
    if matches {
        parsed
    } else {
        Value::String(text.to_string())
    }
}

/// The column a metadata key needs, or `None` when no column can name it.
fn column_name(key: &str) -> Option<String> {
    if key.starts_with("metadata_") {
        if type_suffix(key).is_some() {
            None
        } else {
            Some(format!("metadata_{key}"))
        }
    } else if RESERVED.contains(&key) {
        Some(format!("metadata_{key}"))
    } else {
        Some(key.to_string())
    }
}

/// Whether a value written in a bare column is read back unchanged.
fn infer_safe(value: &MetadataValue) -> bool {
    match value {
        MetadataValue::String(s) => !s.is_empty() && serde_json::from_str::<Value>(s).is_err(),
        _ => true,
    }
}

fn value_cell(value: &MetadataValue) -> String {
    match value {
        MetadataValue::String(s) => s.clone(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

impl OpsDoc for Csv {
    fn parse(self, input: &str) -> Result<Vec<RawTask>, Error> {
        let mut rows = read_records(input)?;
        if rows.is_empty() {
            return Err(Error::syntax("CSV", "the header row is required"));
        }
        let header: Vec<Column> = rows.remove(0).iter().map(|h| classify(h)).collect();

        let mut seen = HashSet::new();
        for column in &header {
            if let Column::Metadata { key, .. } = column
                && !seen.insert(key.clone())
            {
                return Err(Error::Validation(ValidationError::DuplicateMetadataKey(
                    key.clone(),
                )));
            }
        }

        rows.iter().map(|row| row_to_raw(&header, row)).collect()
    }

    fn write(self, tasks: &[Task]) -> Result<String, Error> {
        let rows = records(tasks);

        let mut keys: BTreeSet<String> = BTreeSet::new();
        for row in &rows {
            if let Some(metadata) = &row.metadata {
                keys.extend(metadata.keys().cloned());
            }
        }

        // A key gets a column when every value infers back unchanged; the rest
        // go into the reserved `metadata` JSON column, which the spec allows
        // and the reader merges under the columns.
        let mut columns: Vec<(String, String)> = Vec::new();
        let mut fallback: BTreeSet<String> = BTreeSet::new();
        for key in &keys {
            let values: Vec<&MetadataValue> = rows
                .iter()
                .filter_map(|row| row.metadata.as_ref().and_then(|m| m.get(key)))
                .collect();
            match column_name(key) {
                Some(name) if values.iter().all(|v| infer_safe(v)) => {
                    columns.push((key.clone(), name));
                }
                _ => {
                    fallback.insert(key.clone());
                }
            }
        }
        let split = !fallback.is_empty();

        let mut header: Vec<String> = ["id", "title", "status", "version", "notes", "subtasks"]
            .into_iter()
            .map(str::to_string)
            .collect();
        if split {
            header.push("metadata".to_string());
        }
        for (_, name) in &columns {
            header.push(name.clone());
        }

        let mut out = write_record(&header);
        for row in &rows {
            let mut fields = vec![
                row.id.clone().unwrap_or_default(),
                row.title.clone(),
                row.status.as_str().to_string(),
                row.version.clone().unwrap_or_default(),
                row.notes.clone().unwrap_or_default(),
                serde_json::to_string(&row.subtasks).unwrap_or_else(|_| "[]".to_string()),
            ];
            if split {
                let object: Metadata = row
                    .metadata
                    .as_ref()
                    .map(|m| {
                        m.iter()
                            .filter(|(key, _)| fallback.contains(*key))
                            .map(|(key, value)| (key.clone(), value.clone()))
                            .collect()
                    })
                    .unwrap_or_default();
                fields.push(if object.is_empty() {
                    String::new()
                } else {
                    serde_json::to_string(&object).unwrap_or_default()
                });
            }
            for (key, _) in &columns {
                let value = row.metadata.as_ref().and_then(|m| m.get(key));
                fields.push(value.map(value_cell).unwrap_or_default());
            }
            out.push_str(&write_record(&fields));
        }
        Ok(out)
    }
}

fn row_to_raw(header: &[Column], row: &[String]) -> Result<RawTask, Error> {
    let mut fields = Map::new();
    let mut json_metadata: Option<Map<String, Value>> = None;
    let mut column_metadata: Map<String, Value> = Map::new();

    for (i, column) in header.iter().enumerate() {
        let Some(text) = row.get(i).filter(|text| !text.is_empty()) else {
            continue;
        };
        match column {
            Column::Field("metadata") => match serde_json::from_str::<Value>(text) {
                Ok(Value::Object(map)) => json_metadata = Some(map),
                _ => {
                    return Err(Error::Validation(ValidationError::InvalidFieldType {
                        field: "metadata",
                        expected: "a JSON object",
                    }));
                }
            },
            Column::Field(field) => {
                let value = if *field == "subtasks" {
                    inferred_cell(text)
                } else {
                    Value::String(text.clone())
                };
                fields.insert((*field).to_string(), value);
            }
            Column::Metadata { key, typed } => {
                let value = match typed {
                    Some(ty) => typed_cell(text, *ty),
                    None => inferred_cell(text),
                };
                column_metadata.insert(key.clone(), value);
            }
        }
    }

    // The `metadata` column is merged with the metadata columns; a column
    // that repeats a key already in the object is a duplicate, not an
    // override (RFC-023).
    let mut metadata = json_metadata.unwrap_or_default();
    for (key, value) in column_metadata {
        if metadata.contains_key(&key) {
            return Err(Error::Validation(ValidationError::DuplicateMetadataKey(
                key,
            )));
        }
        metadata.insert(key, value);
    }
    if !metadata.is_empty() {
        fields.insert("metadata".to_string(), Value::Object(metadata));
    }

    RawTask::from_fields(fields)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn records_handle_quoting_and_crlf() {
        let got = read_records("a,b\r\n\"x,y\",\"say \"\"hi\"\"\"\n\"multi\nline\",z\n").unwrap();
        assert_eq!(
            got,
            vec![
                vec!["a".to_string(), "b".to_string()],
                vec!["x,y".to_string(), "say \"hi\"".to_string()],
                vec!["multi\nline".to_string(), "z".to_string()],
            ]
        );
    }

    #[test]
    fn records_reject_an_unterminated_quote() {
        assert!(read_records("\"oops").is_err());
    }

    #[test]
    fn records_drop_a_trailing_blank_line() {
        assert_eq!(read_records("a\n").unwrap(), vec![vec!["a".to_string()]]);
        assert_eq!(read_records("a").unwrap(), vec![vec!["a".to_string()]]);
    }

    #[test]
    fn cells_quote_only_when_needed() {
        assert_eq!(cell("plain"), "plain");
        assert_eq!(cell("a,b"), "\"a,b\"");
        assert_eq!(cell("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(cell("two\nlines"), "\"two\nlines\"");
    }

    #[test]
    fn inference_reads_json_then_falls_back_to_string() {
        assert_eq!(inferred_cell("3"), Value::from(3));
        assert_eq!(inferred_cell("true"), Value::from(true));
        assert_eq!(inferred_cell("high"), Value::from("high"));
        assert_eq!(inferred_cell("2026-08-15"), Value::from("2026-08-15"));
    }

    #[test]
    fn typed_cells_keep_a_string_when_the_type_is_wrong() {
        assert_eq!(typed_cell("3", CellType::Number), Value::from(3));
        assert_eq!(typed_cell("x", CellType::Number), Value::from("x"));
        assert_eq!(typed_cell("true", CellType::Bool), Value::from(true));
        assert_eq!(typed_cell("1", CellType::Bool), Value::from("1"));
        assert_eq!(typed_cell("3", CellType::String), Value::from("3"));
    }

    #[test]
    fn columns_derive_the_key_and_the_type() {
        assert!(matches!(classify("title"), Column::Field("title")));
        assert!(matches!(
            classify("priority"),
            Column::Metadata { typed: None, .. }
        ));
        assert!(matches!(
            classify("metadata_status"),
            Column::Metadata { typed: None, .. }
        ));
        assert!(matches!(
            classify("metadata_estimates_number"),
            Column::Metadata {
                typed: Some(CellType::Number),
                ..
            }
        ));
    }

    #[test]
    fn parse_builds_a_flat_record() {
        let input = "title,status,priority,metadata_status\nTask,open,high,In review\n";
        assert_eq!(Csv.parse(input).unwrap().len(), 1);
    }

    #[test]
    fn parse_rejects_a_duplicate_metadata_key() {
        let err = Csv
            .parse("title,status,priority,metadata_priority\nTask,open,high,low\n")
            .unwrap_err();
        assert_eq!(err.code(), "duplicate-metadata-key");
    }

    #[test]
    fn parse_requires_a_header() {
        assert_eq!(Csv.parse("").unwrap_err().code(), "invalid-syntax");
    }

    #[test]
    fn write_then_parse_preserves_the_model() {
        use crate::v0::doc::helper::forest;
        use crate::v0::model::raw_task::RawTask;

        let input = "id,title,status,subtasks,priority,metadata_status\n\
                     ,Root,open,\"[\"\"a\"\"]\",,\n\
                     a,Child,done,,high,In review\n";
        let raw = Csv.parse(input).unwrap();
        let tasks = forest(
            raw.iter()
                .map(RawTask::to_flat_task)
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
        )
        .unwrap();
        let out = Csv.write(&tasks).unwrap();
        let again = forest(
            Csv.parse(&out)
                .unwrap()
                .iter()
                .map(RawTask::to_flat_task)
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(tasks, again);
    }

    #[test]
    fn a_json_looking_string_falls_back_to_the_metadata_column() {
        use crate::v0::doc::helper::forest;
        use crate::v0::model::raw_task::RawTask;

        let input = "title,status,priority\nTask,open,\"\"\"3\"\"\"\n";
        let raw = Csv.parse(input).unwrap();
        let tasks = forest(
            raw.iter()
                .map(RawTask::to_flat_task)
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
        )
        .unwrap();
        let out = Csv.write(&tasks).unwrap();
        let header = out.lines().next().unwrap();
        assert!(
            header.split(',').any(|column| column == "metadata"),
            "{out}"
        );
        let again = forest(
            Csv.parse(&out)
                .unwrap()
                .iter()
                .map(RawTask::to_flat_task)
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(tasks, again);
    }
}
