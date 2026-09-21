//! Engine adapter boundaries for Nexum.

use nexum_domain::{Progress, TaskId};
use nexum_task::TaskState;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EngineCapabilities {
    pub supports_pause: bool,
    pub supports_resume: bool,
    pub supports_remove: bool,
    pub supports_progress: bool,
}

impl EngineCapabilities {
    pub const BASIC: Self = Self {
        supports_pause: false,
        supports_resume: false,
        supports_remove: false,
        supports_progress: true,
    };
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EngineTask {
    pub task_id: TaskId,
    pub handle: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EngineError {
    UnsupportedOperation(&'static str),
    TaskNotFound(TaskId),
    Failed(String),
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedOperation(operation) => write!(f, "unsupported engine operation: {operation}"),
            Self::TaskNotFound(id) => write!(f, "engine task not found: {id}"),
            Self::Failed(message) => write!(f, "engine error: {message}"),
        }
    }
}

impl std::error::Error for EngineError {}

pub trait EngineAdapter {
    fn name(&self) -> &str;
    fn capabilities(&self) -> EngineCapabilities;
    fn start(&mut self, task_id: &TaskId, source: &str, destination: &str) -> Result<EngineTask, EngineError>;
    fn pause(&mut self, task: &EngineTask) -> Result<(), EngineError>;
    fn resume(&mut self, task: &EngineTask) -> Result<(), EngineError>;
    fn remove(&mut self, task: &EngineTask) -> Result<(), EngineError>;
    fn progress(&self, task: &EngineTask) -> Result<Progress, EngineError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskMapping {
    pub task_id: TaskId,
    pub engine_task: EngineTask,
    pub state: TaskState,
}

impl TaskMapping {
    pub fn new(task_id: TaskId, engine_task: EngineTask) -> Self {
        Self {
            task_id,
            engine_task,
            state: TaskState::Downloading,
        }
    }

    pub fn apply_state(&mut self, state: TaskState) {
        self.state = state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FakeEngine;

    impl EngineAdapter for FakeEngine {
        fn name(&self) -> &str { "fake" }
        fn capabilities(&self) -> EngineCapabilities { EngineCapabilities::BASIC }
        fn start(&mut self, task_id: &TaskId, _source: &str, _destination: &str) -> Result<EngineTask, EngineError> {
            Ok(EngineTask { task_id: task_id.clone(), handle: "fake-1".into() })
        }
        fn pause(&mut self, _task: &EngineTask) -> Result<(), EngineError> {
            Err(EngineError::UnsupportedOperation("pause"))
        }
        fn resume(&mut self, _task: &EngineTask) -> Result<(), EngineError> {
            Err(EngineError::UnsupportedOperation("resume"))
        }
        fn remove(&mut self, _task: &EngineTask) -> Result<(), EngineError> {
            Err(EngineError::UnsupportedOperation("remove"))
        }
        fn progress(&self, _task: &EngineTask) -> Result<Progress, EngineError> {
            Ok(Progress::new(42, Some(100)))
        }
    }

    #[test]
    fn adapter_exposes_capabilities_and_task_mapping() {
        let mut engine = FakeEngine;
        assert_eq!(engine.name(), "fake");
        assert!(engine.capabilities().supports_progress);

        let id = TaskId::from("task-1");
        let task = engine.start(&id, "https://example.com/file", "/tmp/file").unwrap();
        let mapping = TaskMapping::new(id.clone(), task.clone());
        assert_eq!(mapping.task_id, id);
        assert_eq!(mapping.state, TaskState::Downloading);
        assert_eq!(engine.progress(&task).unwrap(), Progress::new(42, Some(100)));
    }

    #[test]
    fn unsupported_operations_are_explicit() {
        let mut engine = FakeEngine;
        let id = TaskId::from("task-1");
        let task = engine.start(&id, "https://example.com/file", "/tmp/file").unwrap();
        assert_eq!(engine.pause(&task), Err(EngineError::UnsupportedOperation("pause")));
    }
}
