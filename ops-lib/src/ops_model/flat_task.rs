use crate::ops_model::status::Status;
use crate::ops_model::task::Task;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlatTask {
    pub title: String,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub metadata: Option<Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub subtasks: Option<Vec<String>>,
}

impl FlatTask {
    pub fn from_task(task: &Task) -> FlatTask {
        FlatTask {
            title: task.title.clone(),
            status: task.status,
            id: task.id.clone(),
            version: task.version.clone(),
            notes: task.notes.clone(),
            metadata: task.metadata.clone(),
            subtasks: Some(task.subtasks.iter().filter_map(|c| c.id.clone()).collect()),
        }
    }

    pub fn child_ids(&self) -> &[String] {
        self.subtasks.as_deref().unwrap_or(&[])
    }
}
