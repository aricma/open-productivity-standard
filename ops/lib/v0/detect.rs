//! Finds the `version` a document declares without knowing any OPS version.
//! The field name is fixed, so a syntax-only scan is enough.

use crate::v0::doc::csv::read_records;
use crate::v0::error::Error;
use crate::v0::format::Format;

/// The version the document declares, or `None` when it declares none.
/// This does not check the version against the released list.
pub(crate) fn detect_version(format: Format, input: &str) -> Result<Option<String>, Error> {
    Ok(match format {
        Format::Json => json_version(input)?,
        Format::Yaml => yaml_version(input)?,
        Format::Jsonl => jsonl_version(input)?,
        Format::Csv => csv_version(input)?,
    })
}

fn value_version(value: &serde_json::Value) -> Option<String> {
    let found = match value {
        serde_json::Value::Object(map) => map.get("version"),
        serde_json::Value::Array(items) => items
            .iter()
            .find_map(|item| item.as_object().and_then(|m| m.get("version"))),
        _ => None,
    };
    match found {
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        _ => None,
    }
}

fn json_version(input: &str) -> Result<Option<String>, Error> {
    let value: serde_json::Value =
        serde_json::from_str(input).map_err(|e| Error::syntax("JSON", e.to_string()))?;
    Ok(value_version(&value))
}

fn yaml_version(input: &str) -> Result<Option<String>, Error> {
    let value: serde_yaml::Value =
        serde_yaml::from_str(input).map_err(|e| Error::syntax("YAML", e.to_string()))?;
    let json = serde_json::to_value(value).map_err(|e| Error::syntax("YAML", e.to_string()))?;
    Ok(value_version(&json))
}

fn jsonl_version(input: &str) -> Result<Option<String>, Error> {
    for line in input.lines().filter(|l| !l.trim().is_empty()) {
        let value: serde_json::Value =
            serde_json::from_str(line).map_err(|e| Error::syntax("JSON", e.to_string()))?;
        if let Some(version) = value_version(&value) {
            return Ok(Some(version));
        }
    }
    Ok(None)
}

fn csv_version(input: &str) -> Result<Option<String>, Error> {
    let rows = read_records(input)?;
    let Some(header) = rows.first() else {
        return Ok(None);
    };
    let Some(column) = header.iter().position(|h| h == "version") else {
        return Ok(None);
    };
    Ok(rows[1..]
        .iter()
        .filter_map(|row| row.get(column))
        .find(|cell| !cell.is_empty())
        .cloned())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    #[test]
    fn finds_the_declared_version_per_format() {
        assert_eq!(
            detect_version(
                Format::Json,
                r#"{"version":"0","title":"t","status":"open"}"#
            )
            .unwrap(),
            Some("0".to_string())
        );
        assert_eq!(
            detect_version(
                Format::Json,
                r#"[{"version":"0","title":"a","status":"open"}]"#
            )
            .unwrap(),
            Some("0".to_string())
        );
        assert_eq!(
            detect_version(Format::Yaml, "version: \"0\"\ntitle: t\nstatus: open\n").unwrap(),
            Some("0".to_string())
        );
        assert_eq!(
            detect_version(
                Format::Jsonl,
                "{\"version\":\"0\",\"title\":\"a\",\"status\":\"open\"}\n"
            )
            .unwrap(),
            Some("0".to_string())
        );
        assert_eq!(
            detect_version(Format::Csv, "title,status,version\na,open,0\n").unwrap(),
            Some("0".to_string())
        );
    }

    #[test]
    fn documents_that_declare_nothing_return_none() {
        assert_eq!(
            detect_version(Format::Json, r#"{"title":"t","status":"open"}"#).unwrap(),
            None
        );
        assert_eq!(detect_version(Format::Json, "[]").unwrap(), None);
    }
}
