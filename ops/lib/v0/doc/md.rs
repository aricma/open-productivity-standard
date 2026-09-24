//! Markdown serialization (tree-preserving): frontmatter carries the root's
//! `version` and `status`; the body holds the heading, checkbox tasks,
//! bulleted metadata, and notes (ADR-011).

use super::helper::{single_root, valid_metadata_key};
use crate::v0::doc::OpsDoc;
use crate::v0::error::{Error, ValidationError};
use crate::v0::model::raw_task::RawTask;
use crate::v0::model::status::Status;
use crate::v0::model::task::Task;
use serde_json::{Map, Value};
use std::collections::HashSet;

pub struct Md;

/// A parsed task, before it becomes a `RawTask`.
#[derive(Default)]
struct Node {
    title: Option<String>,
    done: bool,
    metadata: Map<String, Value>,
    seen_keys: HashSet<String>,
    notes: Vec<String>,
    children: Vec<Node>,
}

impl Node {
    /// The status a checkbox gives a task; `parse` overrides the root's with
    /// the frontmatter value afterwards.
    fn status(&self) -> Option<Status> {
        self.title.as_ref().map(|_| {
            if self.done {
                Status::Done
            } else {
                Status::Open
            }
        })
    }
}

/// The frontmatter YAML and the body, split at the `---` fences.
fn split_frontmatter(input: &str) -> Result<(String, String), Error> {
    let mut lines = input.split('\n');
    match lines.next() {
        Some(first) if first.trim_end_matches('\r') == "---" => {}
        _ => return Err(Error::Md("frontmatter must open with `---`".into())),
    }
    let mut yaml = String::new();
    loop {
        match lines.next() {
            Some(line) if line.trim_end_matches('\r') == "---" => break,
            Some(line) => {
                yaml.push_str(line);
                yaml.push('\n');
            }
            None => return Err(Error::Md("frontmatter must close with `---`".into())),
        }
    }
    Ok((yaml, lines.collect::<Vec<_>>().join("\n")))
}

/// The root's `version` and `status`; any other key is rejected.
fn frontmatter_fields(yaml: &str) -> Result<(Option<Value>, Option<Value>), Error> {
    let value: serde_yaml::Value =
        serde_yaml::from_str(yaml).map_err(|e| Error::Md(e.to_string()))?;
    let mapping = value
        .as_mapping()
        .ok_or_else(|| Error::Md("frontmatter must be a mapping".into()))?;
    let (mut version, mut status) = (None, None);
    for (key, entry) in mapping {
        let key = key
            .as_str()
            .ok_or_else(|| Error::Md("frontmatter keys must be strings".into()))?;
        let entry = serde_json::to_value(entry).map_err(|e| Error::Md(e.to_string()))?;
        match key {
            "version" => version = Some(entry),
            "status" => status = Some(entry),
            other => {
                return Err(Error::Validation(ValidationError::UnexpectedField(
                    other.to_string(),
                )));
            }
        }
    }
    Ok((version, status))
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

/// `- [ ] title` / `- [x] title` into `(done, title)`.
fn parse_checkbox(line: &str) -> Option<(bool, String)> {
    let (done, rest) = if let Some(rest) = line.strip_prefix("- [ ]") {
        (false, rest)
    } else if let Some(rest) = line
        .strip_prefix("- [x]")
        .or_else(|| line.strip_prefix("- [X]"))
    {
        (true, rest)
    } else {
        return None;
    };
    Some((done, rest.trim().to_string()))
}

/// `- key: value` into `(key, value)`; an empty value is absent.
fn metadata_entry(line: &str) -> Option<(String, Option<Value>)> {
    let rest = line.strip_prefix("- ")?;
    let (key, value) = rest.split_once(':')?;
    let key = key.trim();
    if !valid_metadata_key(key) {
        return None;
    }
    let value = value.trim();
    Some((
        key.to_string(),
        if value.is_empty() {
            None
        } else {
            Some(inferred_cell(value))
        },
    ))
}

/// A cell that parses as JSON is that value; anything else is a string.
fn inferred_cell(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.to_string()))
}

/// A metadata value as Markdown: a plain string where that reads back, JSON
/// otherwise (so a string that looks like JSON keeps its type).
fn md_value(value: &Value) -> String {
    match value {
        Value::String(s) if !s.is_empty() && serde_json::from_str::<Value>(s).is_err() => s.clone(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

impl OpsDoc for Md {
    fn parse(self, input: &str) -> Result<Vec<RawTask>, Error> {
        let (yaml, body) = split_frontmatter(input)?;
        let (version, status) = frontmatter_fields(&yaml)?;

        let lines: Vec<(usize, &str)> = body.lines().map(|line| (indent_of(line), line)).collect();

        let mut root = Node::default();
        let mut cursor = 0;
        while cursor < lines.len() && lines[cursor].1.trim().is_empty() {
            cursor += 1;
        }
        if cursor < lines.len()
            && let Some(title) = lines[cursor].1.strip_prefix("# ")
        {
            root.title = Some(title.trim().to_string());
            cursor += 1;
        }
        parse_content(&lines, &mut cursor, 0, &mut root)?;

        let Value::Object(mut fields) = to_raw(&root, true) else {
            return Err(Error::Md("internal: the root is not an object".into()));
        };
        if let Some(version) = version {
            fields.insert("version".to_string(), version);
        }
        if let Some(status) = status {
            fields.insert("status".to_string(), status);
        }
        let raw =
            serde_json::from_value(Value::Object(fields)).map_err(|e| Error::Md(e.to_string()))?;
        Ok(vec![raw])
    }

    fn write(self, tasks: &[Task]) -> Result<String, Error> {
        let root = single_root(tasks, "md")?;
        let mut out = String::from("---\n");
        if let Some(version) = &root.version {
            out.push_str(&format!("version: {version:?}\n"));
        }
        out.push_str(&format!("status: {}\n", root.status.as_str()));
        out.push_str("---\n\n");
        out.push_str(&format!("# {}\n", root.title));
        write_content(&mut out, root, 0);
        Ok(out)
    }
}

/// Reads the content of `node` at `content_indent` until a shallower line.
fn parse_content(
    lines: &[(usize, &str)],
    cursor: &mut usize,
    content_indent: usize,
    node: &mut Node,
) -> Result<(), Error> {
    while *cursor < lines.len() {
        let (indent, line) = lines[*cursor];
        if line.trim().is_empty() {
            *cursor += 1;
            continue;
        }
        if indent < content_indent {
            break;
        }
        let text = &line[content_indent..];
        if indent == content_indent {
            if let Some((done, title)) = parse_checkbox(text) {
                let mut child = Node {
                    title: Some(title),
                    done,
                    ..Default::default()
                };
                *cursor += 1;
                parse_content(lines, cursor, content_indent + 2, &mut child)?;
                node.children.push(child);
                continue;
            }
            if let Some((key, value)) = metadata_entry(text) {
                if !node.seen_keys.insert(key.clone()) {
                    return Err(Error::Validation(ValidationError::DuplicateMetadataKey(
                        key,
                    )));
                }
                if let Some(value) = value {
                    node.metadata.insert(key, value);
                }
                *cursor += 1;
                continue;
            }
        }
        node.notes.push(text.to_string());
        *cursor += 1;
    }
    Ok(())
}

/// A node into the lenient raw JSON shape. `subtasks` becomes nested objects.
/// A root takes its `status` from frontmatter, not from a checkbox.
fn to_raw(node: &Node, root: bool) -> Value {
    let mut fields = Map::new();
    if let Some(title) = &node.title
        && !title.is_empty()
    {
        fields.insert("title".to_string(), Value::String(title.clone()));
    }
    if !root && let Some(status) = node.status() {
        fields.insert(
            "status".to_string(),
            Value::String(status.as_str().to_string()),
        );
    }
    if !node.metadata.is_empty() {
        fields.insert("metadata".to_string(), Value::Object(node.metadata.clone()));
    }
    if !node.notes.is_empty() {
        fields.insert("notes".to_string(), Value::String(node.notes.join("\n")));
    }
    if !node.children.is_empty() {
        fields.insert(
            "subtasks".to_string(),
            Value::Array(node.children.iter().map(|c| to_raw(c, false)).collect()),
        );
    }
    Value::Object(fields)
}

fn write_content(out: &mut String, task: &Task, indent: usize) {
    let pad = " ".repeat(indent);
    if let Some(notes) = &task.notes {
        for line in notes.lines() {
            out.push_str(&pad);
            out.push_str(line);
            out.push('\n');
        }
    }
    if let Some(metadata) = &task.metadata {
        for (key, value) in metadata {
            out.push_str(&format!("{pad}- {key}: {}\n", md_value(value)));
        }
    }
    for child in &task.subtasks {
        write_task(out, child, indent);
    }
}

fn write_task(out: &mut String, task: &Task, indent: usize) {
    let marker = if task.status == Status::Done {
        'x'
    } else {
        ' '
    };
    out.push_str(&format!(
        "{}- [{marker}] {}\n",
        " ".repeat(indent),
        task.title
    ));
    write_content(out, task, indent + 2);
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;

    const SAMPLE: &str = "---\nversion: \"0\"\nstatus: open\n---\n\n# Work\n\nSome notes.\n\n\
- created_at: 2026-01-01T09:00:00Z\n\n- [ ] Parent\n  A note line.\n  - priority: high\n  \
- [x] Child\n    - estimates: 3\n";

    #[test]
    fn frontmatter_splits_from_the_body() {
        let (yaml, body) = split_frontmatter("---\nstatus: open\n---\n\n# Work\n").unwrap();
        assert_eq!(yaml, "status: open\n");
        assert_eq!(body, "\n# Work\n");
    }

    #[test]
    fn frontmatter_requires_both_delimiters() {
        assert!(split_frontmatter("# Work\n").is_err());
        assert!(split_frontmatter("---\nstatus: open\n").is_err());
    }

    #[test]
    fn checkbox_lines_split_marker_and_title() {
        assert_eq!(
            parse_checkbox("- [ ] Buy milk"),
            Some((false, "Buy milk".into()))
        );
        assert_eq!(parse_checkbox("- [x] Done"), Some((true, "Done".into())));
        assert_eq!(parse_checkbox("- [ ]"), Some((false, String::new())));
        assert_eq!(parse_checkbox("- [z] nope"), None);
        assert_eq!(parse_checkbox("- buy milk"), None);
    }

    #[test]
    fn metadata_bullets_need_a_valid_key() {
        assert_eq!(
            metadata_entry("- priority: high"),
            Some(("priority".into(), Some(Value::from("high"))))
        );
        assert_eq!(
            metadata_entry("- estimates: 3"),
            Some(("estimates".into(), Some(Value::from(3))))
        );
        assert_eq!(metadata_entry("- buy milk"), None);
        assert_eq!(metadata_entry("anything: else?"), None);
    }

    #[test]
    fn markdown_values_stay_strings_when_they_look_like_json() {
        assert_eq!(md_value(&Value::from("high")), "high");
        assert_eq!(md_value(&Value::from("3")), "\"3\"");
        assert_eq!(md_value(&Value::from(3)), "3");
        assert_eq!(md_value(&Value::from(true)), "true");
        assert_eq!(md_value(&Value::from(vec!["a", "b"])), r#"["a","b"]"#);
    }

    #[test]
    fn parse_builds_one_root_with_children() {
        let raw = Md.parse(SAMPLE).unwrap();
        assert_eq!(raw.len(), 1);
        let task = raw[0].to_task().unwrap();
        assert_eq!(task.title, "Work");
        assert_eq!(task.version.as_deref(), Some("0"));
        assert_eq!(task.notes.as_deref(), Some("Some notes."));
        assert_eq!(task.subtasks.len(), 1);
        assert_eq!(task.subtasks[0].title, "Parent");
        assert_eq!(task.subtasks[0].notes.as_deref(), Some("A note line."));
        assert_eq!(task.subtasks[0].subtasks[0].status, Status::Done);
    }

    #[test]
    fn parse_rejects_a_duplicate_metadata_key() {
        let input = "---\nstatus: open\n---\n\n# Work\n\n- priority: high\n- priority: low\n";
        assert_eq!(
            Md.parse(input).unwrap_err().code(),
            "duplicate-metadata-key"
        );
    }

    #[test]
    fn read_rejects_a_missing_or_invalid_status() {
        use crate::v0::{self, Format};

        assert_eq!(
            v0::read(Format::Md, "---\nversion: \"0\"\n---\n\n# Work\n")
                .unwrap_err()
                .code(),
            "missing-status"
        );
        assert_eq!(
            v0::read(Format::Md, "---\nstatus: in-progress\n---\n\n# Work\n")
                .unwrap_err()
                .code(),
            "invalid-status"
        );
    }

    #[test]
    fn read_rejects_an_empty_title() {
        use crate::v0::{self, Format};

        assert_eq!(
            v0::read(Format::Md, "---\nstatus: open\n---\n\n# Work\n\n- [ ]\n")
                .unwrap_err()
                .code(),
            "missing-title"
        );
    }

    #[test]
    fn write_then_parse_preserves_the_model() {
        let task = Md.parse(SAMPLE).unwrap()[0].to_task().unwrap();
        let out = Md.write(std::slice::from_ref(&task)).unwrap();
        let again = Md.parse(&out).unwrap()[0].to_task().unwrap();
        assert_eq!(task, again);
    }
}
