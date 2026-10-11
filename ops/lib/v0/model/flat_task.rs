use crate::v0::model::status::Status;
use crate::v0::model::task::Task;
use crate::v0::model::value::{Metadata, absent_metadata, absent_string};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct FlatTask {
    pub title: String,
    pub status: Status,
    #[serde(skip_serializing_if = "absent_string")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "absent_string")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "absent_string")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "absent_metadata")]
    pub metadata: Option<Metadata>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subtasks: Vec<String>,
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
            // An empty id is no id, so it is not a reference either.
            subtasks: task
                .subtasks
                .iter()
                .filter_map(|child| child.id.clone().filter(|id| !id.is_empty()))
                .collect(),
        }
    }

    pub fn child_ids(&self) -> &[String] {
        &self.subtasks
    }
}
