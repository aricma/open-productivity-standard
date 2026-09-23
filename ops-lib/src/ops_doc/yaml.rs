use super::helper::{validate_forest, validated_tasks};
use crate::error::Error;
use crate::ops_doc::OpsDoc;
use crate::ops_doc::single_root;
use crate::ops_model::task::Task;

pub struct Yaml;

impl OpsDoc for Yaml {
    fn parse(self, input: &str) -> Result<Vec<Task>, Error> {
        let root = serde_yaml::from_str(input).map_err(|e| Error::Yaml(e.to_string()))?;
        validated_tasks(vec![root])
    }

    fn write(self, tasks: &[Task]) -> Result<String, Error> {
        validate_forest(tasks).map_err(Error::Validation)?;
        serde_yaml::to_string(single_root(tasks, "yaml")?).map_err(|e| Error::Yaml(e.to_string()))
    }
}
