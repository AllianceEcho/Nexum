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

#[derive(Default)]
pub struct InMemoryEngine {
    tasks: std::collections::HashMap<String, (TaskId, EngineTaskState, Progress)>,
    next_handle: u64,
}

impl InMemoryEngine {
    pub fn new() -> Self { Self::default() }

    fn entry(&self, task: &EngineTask) -> Result<&(TaskId, EngineTaskState, Progress), EngineError> {
        self.tasks.get(&task.handle).ok_or_else(|| EngineError::TaskNotFound(task.task_id.clone()))
    }

    fn entry_mut(&mut self, task: &EngineTask) -> Result<&mut (TaskId, EngineTaskState, Progress), EngineError> {
        self.tasks.get_mut(&task.handle).ok_or_else(|| EngineError::TaskNotFound(task.task_id.clone()))
    }
}

impl EngineAdapter for InMemoryEngine {
    fn name(&self) -> &str { "in-memory" }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            supports_pause: true,
            supports_resume: true,
            supports_remove: true,
            supports_progress: true,
        }
    }

    fn start(&mut self, task_id: &TaskId, _source: &str, _destination: &str) -> Result<EngineTask, EngineError> {
        self.next_handle += 1;
        let handle = format!("memory-{}", self.next_handle);
        self.tasks.insert(
            handle.clone(),
            (task_id.clone(), EngineTaskState::Downloading, Progress::default()),
        );
        Ok(EngineTask { task_id: task_id.clone(), handle })
    }

    fn pause(&mut self, task: &EngineTask) -> Result<(), EngineError> {
        let entry = self.entry_mut(task)?;
        if entry.1 == EngineTaskState::Completed {
            return Err(EngineError::Failed("cannot pause a completed task".into()));
        }
        entry.1 = EngineTaskState::Paused;
        Ok(())
    }

    fn resume(&mut self, task: &EngineTask) -> Result<(), EngineError> {
        let entry = self.entry_mut(task)?;
        if entry.1 != EngineTaskState::Paused {
            return Err(EngineError::Failed("task is not paused".into()));
        }
        entry.1 = EngineTaskState::Downloading;
        Ok(())
    }

    fn remove(&mut self, task: &EngineTask) -> Result<(), EngineError> {
        self.tasks.remove(&task.handle).map(|_| ()).ok_or_else(|| EngineError::TaskNotFound(task.task_id.clone()))
    }

    fn progress(&self, task: &EngineTask) -> Result<Progress, EngineError> {
        Ok(self.entry(task)?.2.clone())
    }

    fn state(&self, task: &EngineTask) -> Result<EngineTaskState, EngineError> {
        Ok(self.entry(task)?.1)
    }
}

pub struct HttpEngine {
    client: reqwest::blocking::Client,
    tasks: std::collections::HashMap<String, (TaskId, EngineTaskState, Progress, String)>,
    next_handle: u64,
}

impl Default for HttpEngine {
    fn default() -> Self {
        Self {
            client: reqwest::blocking::Client::builder()
                .redirect(reqwest::redirect::Policy::limited(5))
                .build()
                .expect("failed to build http client"),
            tasks: std::collections::HashMap::new(),
            next_handle: 0,
        }
    }
}

impl HttpEngine {
    pub fn new() -> Self { Self::default() }

    fn entry(&self, task: &EngineTask) -> Result<&(TaskId, EngineTaskState, Progress, String), EngineError> {
        self.tasks.get(&task.handle).ok_or_else(|| EngineError::TaskNotFound(task.task_id.clone()))
    }

    fn entry_mut(&mut self, task: &EngineTask) -> Result<&mut (TaskId, EngineTaskState, Progress, String), EngineError> {
        self.tasks.get_mut(&task.handle).ok_or_else(|| EngineError::TaskNotFound(task.task_id.clone()))
    }

    fn download(&self, source: &str, destination: &str) -> Result<Progress, EngineError> {
        let mut response = self.client.get(source).send()
            .map_err(|error| EngineError::Failed(error.to_string()))?;
        if !response.status().is_success() {
            return Err(EngineError::Failed(format!("HTTP GET returned {}", response.status())));
        }

        let total = response.content_length();
        let mut file = std::fs::File::create(destination)
            .map_err(|error| EngineError::Failed(error.to_string()))?;
        let mut downloaded = 0u64;
        let mut buffer = [0u8; 32 * 1024];

        loop {
            use std::io::{Read, Write};
            let read = response.read(&mut buffer)
                .map_err(|error| EngineError::Failed(error.to_string()))?;
            if read == 0 { break; }
            file.write_all(&buffer[..read])
                .map_err(|error| EngineError::Failed(error.to_string()))?;
            downloaded += read as u64;
        }

        Ok(Progress::new(downloaded, total))
    }
}

impl EngineAdapter for HttpEngine {
    fn name(&self) -> &str { "http" }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities { supports_pause: false, supports_resume: false, supports_remove: true, supports_progress: true }
    }

    fn start(&mut self, task_id: &TaskId, source: &str, destination: &str) -> Result<EngineTask, EngineError> {
        // Use GET with redirect following (up to 5) instead of separate HEAD+GET
        // This avoids issues where HEAD succeeds but GET follows to a different URL that fails
        let mut response = self.client.get(source).send()
            .map_err(|error| EngineError::Failed(error.to_string()))?;

        if !response.status().is_success() {
            return Err(EngineError::Failed(format!("HTTP GET returned {}", response.status())));
        }

        self.next_handle += 1;
        let handle = format!("http-{}", self.next_handle);
        let total = response.content_length();

        // Verify destination directory exists
        if let Some(parent) = std::path::Path::new(destination).parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let mut file = std::fs::File::create(destination)
            .map_err(|error| EngineError::Failed(error.to_string()))?;

        let mut downloaded = 0u64;
        let mut buffer = [0u8; 32 * 1024];

        loop {
            use std::io::{Read, Write};
            let read = response.read(&mut buffer)
                .map_err(|error| EngineError::Failed(error.to_string()))?;
            if read == 0 { break; }
            file.write_all(&buffer[..read])
                .map_err(|error| EngineError::Failed(error.to_string()))?;
            downloaded += read as u64;
        }

        let progress = Progress::new(downloaded, total);

        // Determine final state based on whether download completed
        let state = if total.map(|t| downloaded >= t).unwrap_or(true) {
            EngineTaskState::Completed
        } else {
            EngineTaskState::Downloading
        };

        self.tasks.insert(
            handle.clone(),
            (task_id.clone(), state, progress, destination.to_owned()),
        );

        Ok(EngineTask { task_id: task_id.clone(), handle })
    }

    fn pause(&mut self, _task: &EngineTask) -> Result<(), EngineError> {
        Err(EngineError::UnsupportedOperation("pause"))
    }

    fn resume(&mut self, _task: &EngineTask) -> Result<(), EngineError> {
        Err(EngineError::UnsupportedOperation("resume"))
    }

    fn remove(&mut self, task: &EngineTask) -> Result<(), EngineError> {
        self.tasks.remove(&task.handle).map(|_| ()).ok_or_else(|| EngineError::TaskNotFound(task.task_id.clone()))
    }

    fn progress(&self, task: &EngineTask) -> Result<Progress, EngineError> {
        Ok(self.entry(task)?.2.clone())
    }

    fn state(&self, task: &EngineTask) -> Result<EngineTaskState, EngineError> {
        Ok(self.entry(task)?.1)
    }
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

    pub fn get_mut(&mut self, name: &str) -> Option<&mut (dyn EngineAdapter + '_)> {
        self.engines.iter_mut().find(|engine| engine.name() == name).map(move |engine| engine.as_mut())
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
    fn http_engine_rejects_invalid_endpoint_without_creating_task() {
        let mut engine = HttpEngine::new();
        let id = TaskId::from("http-task");
        let result = engine.start(&id, "http://127.0.0.1:1/not-found", "/tmp/file");
        assert!(matches!(result, Err(EngineError::Failed(_))));
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
    fn in_memory_engine_controls_task_lifecycle() {
        let mut engine = InMemoryEngine::new();
        let id = TaskId::from("task-1");
        let task = engine.start(&id, "https://example.com/file", "/tmp/file").unwrap();
        assert_eq!(engine.state(&task).unwrap(), EngineTaskState::Downloading);
        engine.pause(&task).unwrap();
        assert_eq!(engine.state(&task).unwrap(), EngineTaskState::Paused);
        engine.resume(&task).unwrap();
        assert_eq!(engine.state(&task).unwrap(), EngineTaskState::Downloading);
        engine.remove(&task).unwrap();
        assert!(matches!(engine.progress(&task), Err(EngineError::TaskNotFound(_))));
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
