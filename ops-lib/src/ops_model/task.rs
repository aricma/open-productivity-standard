use crate::ops_model::status::Status;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

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
    pub metadata: Option<Map<String, Value>>,
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
