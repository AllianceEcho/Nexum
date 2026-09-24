//! Engine adapter boundaries for Nexum.

use nexum_domain::{Progress, TaskId};
use nexum_task::TaskState;
use std::fmt;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

static NEXT_TEMP_DOWNLOAD: AtomicU64 = AtomicU64::new(0);

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
            Self::UnsupportedOperation(operation) => {
                write!(f, "unsupported engine operation: {operation}")
            }
            Self::TaskNotFound(id) => write!(f, "engine task not found: {id}"),
            Self::Failed(message) => write!(f, "engine error: {message}"),
        }
    }
}

impl std::error::Error for EngineError {}

pub trait EngineAdapter {
    fn name(&self) -> &str;
    fn capabilities(&self) -> EngineCapabilities;
    fn start(
        &mut self,
        task_id: &TaskId,
        source: &str,
        destination: &str,
    ) -> Result<EngineTask, EngineError>;
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
    pub fn new() -> Self {
        Self::default()
    }

    fn entry(
        &self,
        task: &EngineTask,
    ) -> Result<&(TaskId, EngineTaskState, Progress), EngineError> {
        self.tasks
            .get(&task.handle)
            .ok_or_else(|| EngineError::TaskNotFound(task.task_id.clone()))
    }

    fn entry_mut(
        &mut self,
        task: &EngineTask,
    ) -> Result<&mut (TaskId, EngineTaskState, Progress), EngineError> {
        self.tasks
            .get_mut(&task.handle)
            .ok_or_else(|| EngineError::TaskNotFound(task.task_id.clone()))
    }
}

impl EngineAdapter for InMemoryEngine {
    fn name(&self) -> &str {
        "in-memory"
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            supports_pause: true,
            supports_resume: true,
            supports_remove: true,
            supports_progress: true,
        }
    }

    fn start(
        &mut self,
        task_id: &TaskId,
        _source: &str,
        _destination: &str,
    ) -> Result<EngineTask, EngineError> {
        self.next_handle += 1;
        let handle = format!("memory-{}", self.next_handle);
        self.tasks.insert(
            handle.clone(),
            (
                task_id.clone(),
                EngineTaskState::Downloading,
                Progress::default(),
            ),
        );
        Ok(EngineTask {
            task_id: task_id.clone(),
            handle,
        })
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
        self.tasks
            .remove(&task.handle)
            .map(|_| ())
            .ok_or_else(|| EngineError::TaskNotFound(task.task_id.clone()))
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

struct TemporaryDownload {
    path: PathBuf,
    file: Option<std::fs::File>,
}

impl TemporaryDownload {
    fn create(destination: &Path) -> Result<Self, EngineError> {
        let parent = destination
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent).map_err(|error| EngineError::Failed(error.to_string()))?;

        for _ in 0..16 {
            let sequence = NEXT_TEMP_DOWNLOAD.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!(
                ".nexum-download-{}-{sequence}.part",
                std::process::id()
            ));
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(file) => {
                    return Ok(Self {
                        path,
                        file: Some(file),
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(EngineError::Failed(error.to_string())),
            }
        }

        Err(EngineError::Failed(
            "could not allocate a temporary download file".into(),
        ))
    }

    fn finish(mut self, destination: &Path) -> Result<(), EngineError> {
        self.file
            .as_ref()
            .expect("temporary download file is open")
            .sync_all()
            .map_err(|error| EngineError::Failed(error.to_string()))?;
        drop(self.file.take());
        std::fs::rename(&self.path, destination)
            .map_err(|error| EngineError::Failed(error.to_string()))
    }
}

impl Drop for TemporaryDownload {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

impl Default for HttpEngine {
    fn default() -> Self {
        Self {
            client: reqwest::blocking::Client::builder()
                .redirect(reqwest::redirect::Policy::limited(5))
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(30 * 60))
                .build()
                .expect("failed to build http client"),
            tasks: std::collections::HashMap::new(),
            next_handle: 0,
        }
    }
}

impl HttpEngine {
    pub fn new() -> Self {
        Self::default()
    }

    fn entry(
        &self,
        task: &EngineTask,
    ) -> Result<&(TaskId, EngineTaskState, Progress, String), EngineError> {
        self.tasks
            .get(&task.handle)
            .ok_or_else(|| EngineError::TaskNotFound(task.task_id.clone()))
    }

    /// Downloads a complete HTTP response before replacing the destination.
    ///
    /// This call blocks and may be run from a worker without holding Core's lock.
    pub fn download_to(&self, source: &str, destination: &str) -> Result<Progress, EngineError> {
        let mut response = self
            .client
            .get(source)
            .send()
            .map_err(|error| EngineError::Failed(error.to_string()))?;
        if !response.status().is_success() {
            return Err(EngineError::Failed(format!(
                "HTTP GET returned {}",
                response.status()
            )));
        }

        let total = response.content_length();
        let destination = Path::new(destination);
        let mut temporary = TemporaryDownload::create(destination)?;
        let mut downloaded = 0u64;
        let mut buffer = [0u8; 32 * 1024];

        loop {
            let read = response
                .read(&mut buffer)
                .map_err(|error| EngineError::Failed(error.to_string()))?;
            if read == 0 {
                break;
            }
            temporary
                .file
                .as_mut()
                .expect("temporary download file is open")
                .write_all(&buffer[..read])
                .map_err(|error| EngineError::Failed(error.to_string()))?;
            downloaded += read as u64;
        }

        if let Some(expected) = total
            && downloaded != expected
        {
            return Err(EngineError::Failed(format!(
                "incomplete HTTP response: expected {expected} bytes, received {downloaded}"
            )));
        }

        temporary.finish(destination)?;
        Ok(Progress::new(downloaded, total))
    }
}

impl EngineAdapter for HttpEngine {
    fn name(&self) -> &str {
        "http"
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            supports_pause: false,
            supports_resume: false,
            supports_remove: true,
            supports_progress: true,
        }
    }

    fn start(
        &mut self,
        task_id: &TaskId,
        source: &str,
        destination: &str,
    ) -> Result<EngineTask, EngineError> {
        let progress = self.download_to(source, destination)?;
        self.next_handle += 1;
        let handle = format!("http-{}", self.next_handle);

        self.tasks.insert(
            handle.clone(),
            (
                task_id.clone(),
                EngineTaskState::Completed,
                progress,
                destination.to_owned(),
            ),
        );

        Ok(EngineTask {
            task_id: task_id.clone(),
            handle,
        })
    }

    fn pause(&mut self, _task: &EngineTask) -> Result<(), EngineError> {
        Err(EngineError::UnsupportedOperation("pause"))
    }

    fn resume(&mut self, _task: &EngineTask) -> Result<(), EngineError> {
        Err(EngineError::UnsupportedOperation("resume"))
    }

    fn remove(&mut self, task: &EngineTask) -> Result<(), EngineError> {
        self.tasks
            .remove(&task.handle)
            .map(|_| ())
            .ok_or_else(|| EngineError::TaskNotFound(task.task_id.clone()))
    }

    fn progress(&self, task: &EngineTask) -> Result<Progress, EngineError> {
        Ok(self.entry(task)?.2.clone())
    }

    fn state(&self, task: &EngineTask) -> Result<EngineTaskState, EngineError> {
        Ok(self.entry(task)?.1)
    }
}

pub struct EngineRegistry {
    engines: Vec<Box<dyn EngineAdapter + Send>>,
}

impl Default for EngineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineRegistry {
    pub fn new() -> Self {
        Self {
            engines: Vec::new(),
        }
    }

    pub fn register(&mut self, engine: Box<dyn EngineAdapter + Send>) {
        self.engines.push(engine);
    }

    pub fn get(&self, name: &str) -> Option<&(dyn EngineAdapter + Send)> {
        self.engines
            .iter()
            .find(|engine| engine.name() == name)
            .map(|engine| engine.as_ref())
    }

    pub fn start_engine(
        &mut self,
        name: &str,
        task_id: &TaskId,
        source: &str,
        destination: &str,
    ) -> Result<EngineTask, EngineError> {
        self.engines
            .iter_mut()
            .find(|engine| engine.name() == name)
            .map(|engine| engine.start(task_id, source, destination))
            .ok_or_else(|| EngineError::Failed(format!("engine not found: {name}")))
            .unwrap_or_else(Err)
    }

    pub fn pause_engine(&mut self, name: &str, task: &EngineTask) -> Result<(), EngineError> {
        self.engines
            .iter_mut()
            .find(|engine| engine.name() == name)
            .ok_or_else(|| EngineError::Failed(format!("engine not found: {name}")))?
            .pause(task)
    }

    pub fn resume_engine(&mut self, name: &str, task: &EngineTask) -> Result<(), EngineError> {
        self.engines
            .iter_mut()
            .find(|engine| engine.name() == name)
            .ok_or_else(|| EngineError::Failed(format!("engine not found: {name}")))?
            .resume(task)
    }

    pub fn remove_engine(&mut self, name: &str, task: &EngineTask) -> Result<(), EngineError> {
        self.engines
            .iter_mut()
            .find(|engine| engine.name() == name)
            .ok_or_else(|| EngineError::Failed(format!("engine not found: {name}")))?
            .remove(task)
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
    use std::net::TcpListener;
    use std::thread;

    static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);

    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "nexum-engine-test-{}-{}",
                std::process::id(),
                NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn serve_once(response: &'static [u8]) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = format!("http://{}/file", listener.local_addr().unwrap());
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut method = [0u8; 3];
            stream.read_exact(&mut method).unwrap();
            assert_eq!(&method, b"GET");
            stream.write_all(response).unwrap();
        });
        (address, server)
    }

    #[derive(Default)]
    struct FakeEngine;

    impl EngineAdapter for FakeEngine {
        fn name(&self) -> &str {
            "fake"
        }
        fn capabilities(&self) -> EngineCapabilities {
            EngineCapabilities::BASIC
        }
        fn start(
            &mut self,
            task_id: &TaskId,
            _source: &str,
            _destination: &str,
        ) -> Result<EngineTask, EngineError> {
            Ok(EngineTask {
                task_id: task_id.clone(),
                handle: "fake-1".into(),
            })
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
    fn http_engine_writes_complete_response_and_reports_completion() {
        let dir = TestDir::new();
        let destination = dir.0.join("nested").join("file.bin");
        let (source, server) =
            serve_once(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nhello");
        let mut engine = HttpEngine::new();

        let task = engine
            .start(
                &TaskId::from("success"),
                &source,
                destination.to_str().unwrap(),
            )
            .unwrap();

        server.join().unwrap();
        assert_eq!(std::fs::read(&destination).unwrap(), b"hello");
        assert_eq!(engine.progress(&task).unwrap(), Progress::new(5, Some(5)));
        assert_eq!(engine.state(&task).unwrap(), EngineTaskState::Completed);
        assert_eq!(
            std::fs::read_dir(destination.parent().unwrap())
                .unwrap()
                .count(),
            1
        );
    }

    #[test]
    fn incomplete_http_response_preserves_existing_destination() {
        let dir = TestDir::new();
        let destination = dir.0.join("file.bin");
        std::fs::write(&destination, b"old bytes").unwrap();
        let (source, server) =
            serve_once(b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n\r\nshort");

        let result = HttpEngine::new().download_to(&source, destination.to_str().unwrap());

        server.join().unwrap();
        assert!(matches!(result, Err(EngineError::Failed(_))));
        assert_eq!(std::fs::read(&destination).unwrap(), b"old bytes");
        assert_eq!(std::fs::read_dir(&dir.0).unwrap().count(), 1);
    }

    #[test]
    fn http_failure_preserves_existing_destination() {
        let dir = TestDir::new();
        let destination = dir.0.join("file.bin");
        std::fs::write(&destination, b"old bytes").unwrap();
        let (source, server) = serve_once(
            b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        );

        let result = HttpEngine::new().download_to(&source, destination.to_str().unwrap());

        server.join().unwrap();
        assert!(matches!(result, Err(EngineError::Failed(_))));
        assert_eq!(std::fs::read(&destination).unwrap(), b"old bytes");
        assert_eq!(std::fs::read_dir(&dir.0).unwrap().count(), 1);
    }

    #[test]
    fn adapter_exposes_capabilities_and_task_mapping() {
        let mut engine = FakeEngine;
        assert_eq!(engine.name(), "fake");
        assert!(engine.capabilities().supports_progress);

        let id = TaskId::from("task-1");
        let task = engine
            .start(&id, "https://example.com/file", "/tmp/file")
            .unwrap();
        let mapping = TaskMapping::new(id.clone(), task.clone());
        assert_eq!(mapping.task_id, id);
        assert_eq!(mapping.state, TaskState::Downloading);
        assert_eq!(
            engine.progress(&task).unwrap(),
            Progress::new(42, Some(100))
        );
        assert_eq!(engine.state(&task).unwrap(), EngineTaskState::Downloading);
        let snapshot = EngineSnapshot::new(
            engine.state(&task).unwrap(),
            engine.progress(&task).unwrap(),
        );
        assert_eq!(map_engine_snapshot(&snapshot).0, TaskState::Downloading);
    }

    #[test]
    fn in_memory_engine_controls_task_lifecycle() {
        let mut engine = InMemoryEngine::new();
        let id = TaskId::from("task-1");
        let task = engine
            .start(&id, "https://example.com/file", "/tmp/file")
            .unwrap();
        assert_eq!(engine.state(&task).unwrap(), EngineTaskState::Downloading);
        engine.pause(&task).unwrap();
        assert_eq!(engine.state(&task).unwrap(), EngineTaskState::Paused);
        engine.resume(&task).unwrap();
        assert_eq!(engine.state(&task).unwrap(), EngineTaskState::Downloading);
        engine.remove(&task).unwrap();
        assert!(matches!(
            engine.progress(&task),
            Err(EngineError::TaskNotFound(_))
        ));
    }

    #[test]
    fn registry_selects_engines_by_name() {
        let mut registry = EngineRegistry::new();
        registry.register(Box::new(FakeEngine));
        assert_eq!(registry.names(), vec!["fake"]);
        assert!(registry.get("fake").is_some());
        assert!(registry.get("missing").is_none());
        assert!(
            registry
                .start_engine("fake", &TaskId::new("x"), "https://x", "/x")
                .is_ok()
        );
    }

    #[test]
    fn engine_snapshot_maps_state_and_progress() {
        let snapshot =
            EngineSnapshot::new(EngineTaskState::Completed, Progress::new(100, Some(100)));
        let (state, progress) = map_engine_snapshot(&snapshot);
        assert_eq!(state, TaskState::Completed);
        assert_eq!(progress, Progress::new(100, Some(100)));
    }

    #[test]
    fn unsupported_operations_are_explicit() {
        let mut engine = FakeEngine;
        let id = TaskId::from("task-1");
        let task = engine
            .start(&id, "https://example.com/file", "/tmp/file")
            .unwrap();
        assert_eq!(
            engine.pause(&task),
            Err(EngineError::UnsupportedOperation("pause"))
        );
    }
}
