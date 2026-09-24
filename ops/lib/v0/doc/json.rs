use crate::v0::doc::OpsDoc;
use crate::v0::doc::helper::single_root;
use crate::v0::error::Error;
use crate::v0::model::raw_task::RawTask;
use crate::v0::model::task::Task;

pub struct Json;

impl OpsDoc for Json {
    fn parse(self, input: &str) -> Result<Vec<RawTask>, Error> {
        let raw: RawTask = serde_json::from_str(input).map_err(|e| Error::Json(e.to_string()))?;
        Ok(vec![raw])
    }

    fn write(self, tasks: &[Task]) -> Result<String, Error> {
        serde_json::to_string_pretty(single_root(tasks, "json")?)
            .map_err(|e| Error::Json(e.to_string()))
    }
}
