//! JSONL serialization (flat/streamable forest).

use super::helper::records;
use crate::v0::doc::OpsDoc;
use crate::v0::error::Error;
use crate::v0::format::Format;
use crate::v0::limits::{Limit, check};
use crate::v0::model::raw_task::RawTask;
use crate::v0::model::task::Task;

pub struct Jsonl;

impl OpsDoc for Jsonl {
    fn parse(self, input: &str) -> Result<Vec<RawTask>, Error> {
        let mut raws = Vec::new();
        for (idx, line) in input.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let raw: RawTask = serde_json::from_str(line)
                .map_err(|e| Error::syntax(Format::Jsonl, format!("line {}: {e}", idx + 1)))?;
            raws.push(raw);
        }
        Ok(raws)
    }

    fn write(self, tasks: &[Task]) -> Result<String, Error> {
        let mut out = String::new();
        for r in records(tasks) {
            out.push_str(
                &serde_json::to_string(&r)
                    .map_err(|e| Error::syntax(Format::Jsonl, e.to_string()))?,
            );
            out.push('\n');
            // The facade checks the finished document too; this stops a
            // runaway document from being built in the first place.
            check(Limit::OutputBytes, out.len()).map_err(Error::Validation)?;
        }
        Ok(out)
    }
}
