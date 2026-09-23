use super::helper::{validate_forest, validated_tasks};
use super::raw::{self, RawTask};
use crate::doc::OpsDoc;
use crate::doc::single_root;
use crate::error::Error;
use crate::model::task::Task;

pub struct Yaml;

impl OpsDoc for Yaml {
    fn parse(self, input: &str) -> Result<Vec<Task>, Error> {
        let raw: RawTask = serde_yaml::from_str(input).map_err(|e| Error::Yaml(e.to_string()))?;
        validated_tasks(vec![raw::build_task(raw)?])
    }

    fn write(self, tasks: &[Task]) -> Result<String, Error> {
        validate_forest(tasks).map_err(Error::Validation)?;
        serde_yaml::to_string(single_root(tasks, "yaml")?).map_err(|e| Error::Yaml(e.to_string()))
    }
}
