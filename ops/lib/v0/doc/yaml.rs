use crate::v0::doc::OpsDoc;
use crate::v0::doc::helper::single_root;
use crate::v0::error::Error;
use crate::v0::model::raw_task::RawTask;
use crate::v0::model::task::Task;

pub struct Yaml;

impl OpsDoc for Yaml {
    fn parse(self, input: &str) -> Result<Vec<RawTask>, Error> {
        let raw: RawTask = serde_yaml::from_str(input).map_err(|e| Error::Yaml(e.to_string()))?;
        Ok(vec![raw])
    }

    fn write(self, tasks: &[Task]) -> Result<String, Error> {
        serde_yaml::to_string(single_root(tasks, "yaml")?).map_err(|e| Error::Yaml(e.to_string()))
    }
}
