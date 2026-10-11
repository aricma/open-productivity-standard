use crate::v0::model::status::Status;
use crate::v0::model::value::Metadata;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub title: String,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub metadata: Option<Metadata>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
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
