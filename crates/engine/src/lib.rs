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
    fn state(&self, task: &EngineTask) -> Result<EngineTaskState, EngineError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EngineTaskState {
    Queued,
    Downloading,
    Paused,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EngineSnapshot {
    pub state: EngineTaskState,
    pub progress: Progress,
}

impl EngineSnapshot {
    pub fn new(state: EngineTaskState, progress: Progress) -> Self {
        Self { state, progress }
    }

    pub fn to_task_state(&self) -> TaskState {
        match self.state {
            EngineTaskState::Queued => TaskState::Queued,
            EngineTaskState::Downloading => TaskState::Downloading,
            EngineTaskState::Paused => TaskState::Paused,
            EngineTaskState::Completed => TaskState::Completed,
            EngineTaskState::Failed => TaskState::Failed,
        }
    }
}

pub fn map_engine_snapshot(snapshot: &EngineSnapshot) -> (TaskState, Progress) {
    (snapshot.to_task_state(), snapshot.progress.clone())
}

pub struct EngineRegistry {
    engines: Vec<Box<dyn EngineAdapter>>,
}

impl Default for EngineRegistry {
    fn default() -> Self { Self::new() }
}

impl EngineRegistry {
    pub fn new() -> Self { Self { engines: Vec::new() } }

    pub fn register(&mut self, engine: Box<dyn EngineAdapter>) {
        self.engines.push(engine);
    }

    pub fn get(&self, name: &str) -> Option<&dyn EngineAdapter> {
        self.engines.iter().find(|engine| engine.name() == name).map(|engine| engine.as_ref())
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut dyn EngineAdapter> {
        self.engines.iter_mut().find(|engine| engine.name() == name).map(|engine| engine.as_mut())
    }

    pub fn names(&self) -> Vec<&str> {
        self.engines.iter().map(|engine| engine.name()).collect()
    }
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
        fn state(&self, _task: &EngineTask) -> Result<EngineTaskState, EngineError> {
            Ok(EngineTaskState::Downloading)
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
        assert_eq!(engine.state(&task).unwrap(), EngineTaskState::Downloading);
        let snapshot = EngineSnapshot::new(engine.state(&task).unwrap(), engine.progress(&task).unwrap());
        assert_eq!(map_engine_snapshot(&snapshot).0, TaskState::Downloading);
    }

    #[test]
    fn registry_selects_engines_by_name() {
        let mut registry = EngineRegistry::new();
        registry.register(Box::new(FakeEngine));
        assert_eq!(registry.names(), vec!["fake"]);
        assert!(registry.get("fake").is_some());
        assert!(registry.get("missing").is_none());
        assert!(registry.get_mut("fake").is_some());
    }

    #[test]
    fn engine_snapshot_maps_state_and_progress() {
        let snapshot = EngineSnapshot::new(EngineTaskState::Completed, Progress::new(100, Some(100)));
        let (state, progress) = map_engine_snapshot(&snapshot);
        assert_eq!(state, TaskState::Completed);
        assert_eq!(progress, Progress::new(100, Some(100)));
    }

    #[test]
    fn unsupported_operations_are_explicit() {
        let mut engine = FakeEngine;
        let id = TaskId::from("task-1");
        let task = engine.start(&id, "https://example.com/file", "/tmp/file").unwrap();
        assert_eq!(engine.pause(&task), Err(EngineError::UnsupportedOperation("pause")));
    }
}
