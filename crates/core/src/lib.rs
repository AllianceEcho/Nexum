//! Nexum Core orchestration layer.

pub use nexum_domain;
pub use nexum_engine;
pub use nexum_resolver;
pub use nexum_scheduler;
pub use nexum_scheduler::SchedulerConfig;
pub use nexum_storage;
pub use nexum_task;

use nexum_domain::{Destination, DownloadSource, TaskId};
use nexum_engine::{EngineError, EngineRegistry, EngineTask};
use nexum_resolver::{ResolveRequest, ResolveResult, ResolverError, ResolverRegistry};
use nexum_scheduler::{Priority, Scheduler, SchedulerError, SchedulerEvent};
use nexum_storage::{InMemoryRepository, StorageError, StoredTask, TaskRepository};
use nexum_task::{DownloadTask, TaskService, TaskServiceError, TaskState};
use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
pub enum CoreError {
    Scheduler(SchedulerError),
    Task(TaskServiceError),
    Storage(StorageError),
    Resolver(ResolverError),
    Engine(EngineError),
}

impl From<SchedulerError> for CoreError {
    fn from(value: SchedulerError) -> Self {
        Self::Scheduler(value)
    }
}
impl From<TaskServiceError> for CoreError {
    fn from(value: TaskServiceError) -> Self {
        Self::Task(value)
    }
}
impl From<StorageError> for CoreError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}
impl From<ResolverError> for CoreError {
    fn from(value: ResolverError) -> Self {
        Self::Resolver(value)
    }
}
impl From<EngineError> for CoreError {
    fn from(value: EngineError) -> Self {
        Self::Engine(value)
    }
}

/// Core orchestration with an injectable task repository.
pub struct Core<R: TaskRepository = InMemoryRepository> {
    pub tasks: TaskService,
    pub scheduler: Scheduler,
    pub repository: R,
    pub resolver: ResolverRegistry,
    pub engines: EngineRegistry,
    pub engine_tasks: HashMap<TaskId, (String, EngineTask)>,
}

impl Core<InMemoryRepository> {
    pub fn new(scheduler_config: SchedulerConfig) -> Result<Self, SchedulerError> {
        Self::with_repository(scheduler_config, InMemoryRepository::new())
    }
}

impl<R: TaskRepository> Core<R> {
    pub fn with_repository(
        scheduler_config: SchedulerConfig,
        repository: R,
    ) -> Result<Self, SchedulerError> {
        Ok(Self {
            tasks: TaskService::new(),
            scheduler: Scheduler::new(scheduler_config)?,
            repository,
            resolver: ResolverRegistry::new(),
            engines: {
                let mut registry = EngineRegistry::new();
                registry.register(Box::new(nexum_engine::InMemoryEngine::new()));
                registry.register(Box::new(nexum_engine::HttpEngine::new()));
                registry
            },
            engine_tasks: HashMap::new(),
        })
    }

    pub fn resolve_source(&self, source: impl Into<String>) -> Result<ResolveResult, CoreError> {
        let request = ResolveRequest::new(source)?;
        Ok(self.resolver.resolve(&request)?)
    }

    pub fn create_task(
        &mut self,
        id: impl Into<TaskId>,
        source: DownloadSource,
        destination: Destination,
    ) -> Result<DownloadTask, CoreError> {
        self.resolve_source(source.as_str())?;
        let task = self.tasks.create(id, source, destination)?.clone();
        self.repository.insert(StoredTask::from(task.clone()))?;
        Ok(task)
    }

    pub fn queue_task(&mut self, id: &TaskId, priority: Priority) -> Result<(), CoreError> {
        self.scheduler.enqueue(&mut self.tasks, id, priority)?;
        self.persist_task(id)
    }

    pub fn start_next(&mut self) -> Result<Option<TaskId>, CoreError> {
        self.start_next_with_engine("in-memory")
    }

    pub fn start_next_with_engine(
        &mut self,
        engine_name: &str,
    ) -> Result<Option<TaskId>, CoreError> {
        let id = self.scheduler.start_next(&mut self.tasks)?;
        let Some(task_id) = id else {
            return Ok(None);
        };
        let task = self
            .tasks
            .get(&task_id)
            .cloned()
            .ok_or_else(|| TaskServiceError::NotFound(task_id.clone()))?;
        let result = self.engines.start_engine(
            engine_name,
            &task.id,
            task.source.as_str(),
            task.destination.as_str(),
        );
        match result {
            Ok(engine_task) => {
                self.engine_tasks
                    .insert(task_id.clone(), (engine_name.to_owned(), engine_task));
                self.sync_engine_task(&task_id)?;
                self.persist_task(&task_id)?;
                Ok(Some(task_id))
            }
            Err(error) => {
                self.scheduler
                    .mark_finished(&mut self.tasks, &task_id, TaskState::Failed)?;
                self.persist_task(&task_id)?;
                Err(CoreError::Engine(error))
            }
        }
    }

    pub fn sync_engine_task(&mut self, id: &TaskId) -> Result<(), CoreError> {
        let (engine_name, engine_task) = self
            .engine_tasks
            .get(id)
            .ok_or_else(|| EngineError::TaskNotFound(id.clone()))?
            .clone();
        let snapshot = match self.engines.get(&engine_name) {
            Some(engine) => nexum_engine::EngineSnapshot::new(
                engine.state(&engine_task)?,
                engine.progress(&engine_task)?,
            ),
            None => {
                return Err(EngineError::Failed(format!("engine not found: {engine_name}")).into());
            }
        };
        let (state, progress) = nexum_engine::map_engine_snapshot(&snapshot);
        self.tasks.update_progress(id, progress)?;
        if self.tasks.get(id).map(|task| task.state) != Some(state) {
            self.tasks.transition(id, state)?;
        }
        self.persist_task(id)
    }

    pub fn pause_task(&mut self, id: &TaskId) -> Result<(), CoreError> {
        if let Some((engine_name, engine_task)) = self.engine_tasks.get(id).cloned() {
            self.engines.pause_engine(&engine_name, &engine_task)?;
        }
        self.scheduler.pause(&mut self.tasks, id)?;
        self.persist_task(id)
    }

    pub fn resume_task(&mut self, id: &TaskId) -> Result<bool, CoreError> {
        if let Some((engine_name, engine_task)) = self.engine_tasks.get(id).cloned() {
            self.engines.resume_engine(&engine_name, &engine_task)?;
        }
        let resumed = self.scheduler.resume(&mut self.tasks, id)?;
        if resumed {
            self.persist_task(id)?;
        }
        Ok(resumed)
    }

    pub fn finish_task(&mut self, id: &TaskId, state: TaskState) -> Result<(), CoreError> {
        self.scheduler.mark_finished(&mut self.tasks, id, state)?;
        self.persist_task(id)
    }

    pub fn update_progress(
        &mut self,
        id: &TaskId,
        progress: nexum_domain::Progress,
    ) -> Result<(), CoreError> {
        self.tasks.update_progress(id, progress)?;
        self.persist_task(id)
    }

    pub fn remove_task(&mut self, id: &TaskId) -> Result<DownloadTask, CoreError> {
        if let Some((engine_name, engine_task)) = self.engine_tasks.remove(id) {
            self.engines.remove_engine(&engine_name, &engine_task)?;
        }
        let task = self.tasks.remove(id)?;
        self.repository.remove(id)?;
        Ok(task)
    }

    pub fn persist_task(&mut self, id: &TaskId) -> Result<(), CoreError> {
        let task = self
            .tasks
            .get(id)
            .ok_or_else(|| StorageError::NotFound(id.clone()))?
            .clone();
        self.repository.update(StoredTask::from(task))?;
        Ok(())
    }

    /// Loads persisted tasks and reconstructs the in-memory queue.
    ///
    /// Downloading, Paused, and Retrying tasks are recovered as Queued so
    /// restart never falsely reports an active transfer.
    pub fn recover(&mut self) -> Result<usize, CoreError> {
        let stored_tasks = self.repository.list()?;
        let mut restored = 0;

        for stored in stored_tasks {
            if self.tasks.get(&stored.id).is_some() {
                continue;
            }

            let task = stored.into_task();
            let id = task.id.clone();
            let state = task.state;
            self.tasks.restore(task)?;

            match state {
                TaskState::Queued
                | TaskState::Downloading
                | TaskState::Paused
                | TaskState::Retrying => {
                    if state != TaskState::Queued {
                        self.tasks.transition(&id, TaskState::Queued)?;
                    }
                    self.scheduler.restore_queued(&id, Priority::NORMAL);
                }
                TaskState::Created | TaskState::Completed | TaskState::Failed => {}
            }

            restored += 1;
        }

        Ok(restored)
    }

    pub fn drain_task_events(&mut self) -> Vec<nexum_task::TaskEvent> {
        self.tasks.drain_events()
    }

    pub fn drain_scheduler_events(&mut self) -> Vec<SchedulerEvent> {
        self.scheduler.drain_events()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexum_domain::{Destination, DownloadSource, Progress};
    use nexum_storage::SqliteRepository;

    fn config() -> SchedulerConfig {
        SchedulerConfig {
            max_concurrent_tasks: 2,
            ..SchedulerConfig::default()
        }
    }

    fn task_source() -> (DownloadSource, Destination) {
        (
            DownloadSource::new("https://example.com/file"),
            Destination::new("/tmp/file"),
        )
    }

    #[test]
    fn resolves_sources_through_core_boundary() {
        let core = Core::with_repository(config(), InMemoryRepository::new()).unwrap();
        let result = core.resolve_source("https://example.com/file").unwrap();
        assert_eq!(result.kind, nexum_resolver::ResolveKind::Https);
        let (source, destination) = task_source();
        let mut core = core;
        let id = TaskId::from("task-1");
        core.create_task(id.clone(), source, destination).unwrap();
        core.queue_task(&id, Priority::HIGH).unwrap();
        core.update_progress(&id, Progress::new(128, Some(1024)))
            .unwrap();

        let stored = core.repository.get(&id).unwrap().unwrap();
        assert_eq!(stored.state, TaskState::Queued);
        assert_eq!(stored.progress, Progress::new(128, Some(1024)));
    }

    #[test]
    fn rejects_task_creation_with_unsupported_source() {
        let mut core = Core::with_repository(config(), InMemoryRepository::new()).unwrap();
        let error = core
            .create_task(
                "invalid-source",
                DownloadSource::new("ftp://example.com/file"),
                Destination::new("/tmp/file"),
            )
            .unwrap_err();
        assert!(matches!(
            error,
            CoreError::Resolver(nexum_resolver::ResolverError::UnsupportedScheme(_))
        ));
        assert!(core.tasks.get(&TaskId::from("invalid-source")).is_none());
    }

    #[test]
    fn recovery_rebuilds_queued_tasks() {
        let mut core = Core::with_repository(config(), InMemoryRepository::new()).unwrap();
        let (source, destination) = task_source();
        let id = TaskId::from("task-1");
        core.create_task(id.clone(), source, destination).unwrap();
        core.queue_task(&id, Priority::NORMAL).unwrap();

        let repository = core.repository;
        let mut restarted = Core::with_repository(config(), repository).unwrap();
        assert_eq!(restarted.recover().unwrap(), 1);
        assert_eq!(restarted.tasks.get(&id).unwrap().state, TaskState::Queued);
        assert_eq!(restarted.scheduler.queued_len(), 1);
    }

    #[test]
    fn sqlite_repository_can_be_injected_into_core() {
        let mut core =
            Core::with_repository(config(), SqliteRepository::open_in_memory().unwrap()).unwrap();
        let (source, destination) = task_source();
        let id = TaskId::from("sqlite-task");
        core.create_task(id.clone(), source, destination).unwrap();
        core.queue_task(&id, Priority::NORMAL).unwrap();
        core.start_next().unwrap();
        core.update_progress(&id, Progress::new(64, Some(100)))
            .unwrap();

        let stored = core.repository.get(&id).unwrap().unwrap();
        assert_eq!(stored.state, TaskState::Downloading);
        assert_eq!(stored.progress.downloaded_bytes, 64);
    }

    #[test]
    fn full_task_lifecycle() {
        let mut core = Core::with_repository(config(), InMemoryRepository::new()).unwrap();
        let (source, destination) = task_source();
        let id = TaskId::from("lifecycle-task");

        // Create
        let task = core
            .create_task(id.clone(), source.clone(), destination.clone())
            .unwrap();
        assert_eq!(task.state, TaskState::Created);

        // Queue
        core.queue_task(&id, Priority::NORMAL).unwrap();
        assert_eq!(core.tasks.get(&id).unwrap().state, TaskState::Queued);

        // Start (in-memory engine completes immediately)
        let started_id = core.start_next().unwrap();
        assert_eq!(started_id, Some(id.clone()));
        assert_eq!(core.tasks.get(&id).unwrap().state, TaskState::Downloading);

        // Update progress
        core.update_progress(&id, Progress::new(512, Some(1024)))
            .unwrap();
        let stored = core.repository.get(&id).unwrap().unwrap();
        assert_eq!(stored.progress.downloaded_bytes, 512);
        assert_eq!(stored.progress.total_bytes, Some(1024));

        // Pause
        core.pause_task(&id).unwrap();
        assert_eq!(core.tasks.get(&id).unwrap().state, TaskState::Paused);

        // Resume
        core.resume_task(&id).unwrap();
        assert_eq!(core.tasks.get(&id).unwrap().state, TaskState::Downloading);

        // Complete (simulate engine completing)
        core.finish_task(&id, TaskState::Completed).unwrap();
        assert!(core.tasks.get(&id).unwrap().is_terminal());
    }

    #[test]
    fn concurrent_tasks_respect_max_concurrent_limit() {
        let mut core = Core::with_repository(config(), InMemoryRepository::new()).unwrap(); // max_concurrent_tasks: 2
        let (source, destination) = task_source();

        let id1 = TaskId::from("concurrent-1");
        let id2 = TaskId::from("concurrent-2");
        let id3 = TaskId::from("concurrent-3");

        core.create_task(id1.clone(), source.clone(), destination.clone())
            .unwrap();
        core.create_task(id2.clone(), source.clone(), destination.clone())
            .unwrap();
        core.create_task(id3.clone(), source, destination).unwrap();

        core.queue_task(&id1, Priority::NORMAL).unwrap();
        core.queue_task(&id2, Priority::NORMAL).unwrap();
        core.queue_task(&id3, Priority::HIGH).unwrap();

        // Start first two (max_concurrent_tasks = 2)
        let r1 = core.start_next().unwrap();
        let r2 = core.start_next().unwrap();
        assert!(r1.is_some());
        assert!(r2.is_some());

        // Third should not start (limit reached)
        let r3 = core.start_next().unwrap();
        assert!(r3.is_none());

        // Finish one to free a slot
        let task1 = core.tasks.get(&id1).unwrap().clone();
        core.finish_task(&task1.id, TaskState::Completed).unwrap();

        // Now the third (HIGH priority) should start
        let r3 = core.start_next().unwrap();
        assert!(r3.is_some());
        assert_eq!(r3.unwrap(), id3);
    }

    #[test]
    fn core_drains_events() {
        let mut core = Core::with_repository(config(), InMemoryRepository::new()).unwrap();
        let (source, destination) = task_source();
        let id = TaskId::from("events-task");
        core.create_task(id.clone(), source, destination).unwrap();

        let events = core.drain_task_events();
        assert!(!events.is_empty());
        assert_eq!(
            events[0],
            nexum_task::TaskEvent::Created {
                task_id: id.clone()
            }
        );

        // After draining, events should be empty
        assert!(core.drain_task_events().is_empty());
    }

    #[test]
    fn core_schedulers_drain_scheduler_events() {
        let mut core = Core::with_repository(config(), InMemoryRepository::new()).unwrap();
        let (source, destination) = task_source();
        let id = TaskId::from("sched-events");
        core.create_task(id.clone(), source, destination).unwrap();
        core.queue_task(&id, Priority::NORMAL).unwrap();

        let events = core.drain_scheduler_events();
        assert!(!events.is_empty());
        assert!(matches!(
            &events[0],
            nexum_scheduler::SchedulerEvent::Enqueued { .. }
        ));
    }

    #[test]
    fn core_persists_and_recovers_full_lifecycle() {
        let mut core =
            Core::with_repository(config(), SqliteRepository::open_in_memory().unwrap()).unwrap();
        let (source, destination) = task_source();
        let id = TaskId::from("persist-recover");

        core.create_task(id.clone(), source.clone(), destination.clone())
            .unwrap();
        core.queue_task(&id, Priority::NORMAL).unwrap();

        // Save repository and create fresh core
        let repository = core.repository;
        let mut fresh = Core::with_repository(config(), repository).unwrap();

        // Recover should restore the queued task
        let restored = fresh.recover().unwrap();
        assert_eq!(restored, 1);
        assert_eq!(fresh.tasks.get(&id).unwrap().state, TaskState::Queued);
        assert_eq!(fresh.scheduler.queued_len(), 1);

        // Start and pause should also persist
        fresh.start_next().unwrap();
        fresh.pause_task(&id).unwrap();

        // After another restart, task should be recovered as queued (not downloading/paused)
        let repository2 = fresh.repository;
        let mut restarted = Core::with_repository(config(), repository2).unwrap();
        assert_eq!(restarted.recover().unwrap(), 1);
        // State is normalized to Queued during recovery
        assert_eq!(restarted.tasks.get(&id).unwrap().state, TaskState::Queued);
    }

    #[test]
    fn core_removes_task_from_all_subsystems() {
        let mut core = Core::with_repository(config(), InMemoryRepository::new()).unwrap();
        let (source, destination) = task_source();
        let id = TaskId::from("remove-task");

        core.create_task(id.clone(), source, destination).unwrap();
        core.queue_task(&id, Priority::NORMAL).unwrap();
        core.start_next().unwrap();

        // Remove should clean up all subsystems
        let removed = core.remove_task(&id).unwrap();
        assert_eq!(removed.id, id);
        assert!(core.tasks.get(&id).is_none());
        assert!(core.repository.get(&id).unwrap().is_none());
        assert!(core.engine_tasks.is_empty());
    }
}
