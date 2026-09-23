use super::helper::{validate_forest, validated_tasks};
use crate::error::Error;
use crate::ops_doc::OpsDoc;
use crate::ops_doc::single_root;
use crate::ops_model::task::Task;

pub struct Json;

impl OpsDoc for Json {
    fn parse(self, input: &str) -> Result<Vec<Task>, Error> {
        let root = serde_json::from_str(input).map_err(|e| Error::Json(e.to_string()))?;
        validated_tasks(vec![root])
    }

    fn write(self, tasks: &[Task]) -> Result<String, Error> {
        validate_forest(tasks).map_err(Error::Validation)?;
        serde_json::to_string_pretty(single_root(tasks, "json")?)
            .map_err(|e| Error::Json(e.to_string()))
    }
}
