use super::error_kind::ErrorKind;
use ops_lib::Task;

#[derive(Debug, Clone)]
pub enum Expectation {
    Model(Vec<Task>),
    Error(ErrorKind),
}
