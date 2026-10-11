use crate::v0::doc::OpsDoc;
use crate::v0::error::Error;
use crate::v0::model::raw_task::RawTask;
use crate::v0::model::task::Task;
use serde_json::Value;

pub struct Json;

impl OpsDoc for Json {
    fn parse(self, input: &str) -> Result<Vec<RawTask>, Error> {
        let value: Value =
            serde_json::from_str(input).map_err(|e| Error::syntax("JSON", e.to_string()))?;
        match value {
            Value::Array(items) => items
                .into_iter()
                .map(|item| {
                    serde_json::from_value::<RawTask>(item)
                        .map_err(|e| Error::syntax("JSON", e.to_string()))
                })
                .collect(),
            other => serde_json::from_value::<RawTask>(other)
                .map(|raw| vec![raw])
                .map_err(|e| Error::syntax("JSON", e.to_string())),
        }
    }

    fn write(self, tasks: &[Task]) -> Result<String, Error> {
        let value = if let [task] = tasks {
            serde_json::to_value(task).map_err(|e| Error::syntax("JSON", e.to_string()))?
        } else {
            serde_json::to_value(tasks).map_err(|e| Error::syntax("JSON", e.to_string()))?
        };
        serde_json::to_string_pretty(&value).map_err(|e| Error::syntax("JSON", e.to_string()))
    }
}
