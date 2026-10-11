use crate::v0::doc::OpsDoc;
use crate::v0::error::Error;
use crate::v0::format::Format;
use crate::v0::model::raw_task::RawTask;
use crate::v0::model::task::Task;
use serde_yaml::Value;

pub struct Yaml;

impl OpsDoc for Yaml {
    fn parse(self, input: &str) -> Result<Vec<RawTask>, Error> {
        let value: Value =
            serde_yaml::from_str(input).map_err(|e| Error::syntax(Format::Yaml, e.to_string()))?;
        match value {
            Value::Sequence(items) => items
                .into_iter()
                .map(|item| {
                    let json = serde_json::to_value(item)
                        .map_err(|e| Error::syntax(Format::Yaml, e.to_string()))?;
                    serde_json::from_value::<RawTask>(json)
                        .map_err(|e| Error::syntax(Format::Yaml, e.to_string()))
                })
                .collect(),
            other => {
                let json = serde_json::to_value(other)
                    .map_err(|e| Error::syntax(Format::Yaml, e.to_string()))?;
                serde_json::from_value::<RawTask>(json)
                    .map(|raw| vec![raw])
                    .map_err(|e| Error::syntax(Format::Yaml, e.to_string()))
            }
        }
    }

    fn write(self, tasks: &[Task]) -> Result<String, Error> {
        let value = if let [task] = tasks {
            serde_yaml::to_value(task).map_err(|e| Error::syntax(Format::Yaml, e.to_string()))?
        } else {
            serde_yaml::to_value(tasks).map_err(|e| Error::syntax(Format::Yaml, e.to_string()))?
        };
        serde_yaml::to_string(&value).map_err(|e| Error::syntax(Format::Yaml, e.to_string()))
    }
}
