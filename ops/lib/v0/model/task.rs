use crate::v0::model::status::Status;
use crate::v0::model::value::{Metadata, absent_metadata, absent_string};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Default, Serialize)]
pub struct Task {
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
    pub subtasks: Vec<Task>,
}

impl Task {
    pub fn new(title: String, status: Status) -> Task {
        Task {
            title,
            status,
            ..Default::default()
        }
    }
}
