//! Nexum Core orchestration layer.

pub use nexum_domain;
pub use nexum_scheduler;
pub use nexum_storage;
pub use nexum_task;
pub use nexum_resolver;

use nexum_domain::{Destination, DownloadSource, TaskId};
use nexum_scheduler::{Priority, Scheduler, SchedulerConfig, SchedulerError, SchedulerEvent};
use nexum_storage::{InMemoryRepository, StorageError, StoredTask, TaskRepository};
use nexum_resolver::{ResolveRequest, ResolveResult, ResolverError, ResolverRegistry};
use nexum_task::{DownloadTask, TaskService, TaskServiceError, TaskState};

#[derive(Debug, Eq, PartialEq)]
pub enum CoreError {
    Scheduler(SchedulerError),
    Task(TaskServiceError),
    Storage(StorageError),
    Resolver(ResolverError),
}

impl From<SchedulerError> for CoreError {
    fn from(value: SchedulerError) -> Self { Self::Scheduler(value) }
}
impl From<TaskServiceError> for CoreError {
    fn from(value: TaskServiceError) -> Self { Self::Task(value) }
}
impl From<StorageError> for CoreError {
    fn from(value: StorageError) -> Self { Self::Storage(value) }
}
impl From<ResolverError> for CoreError {
    fn from(value: ResolverError) -> Self { Self::Resolver(value) }
}

/// Core orchestration with an injectable task repository.
pub struct Core<R: TaskRepository = InMemoryRepository> {
    pub tasks: TaskService,
    pub scheduler: Scheduler,
    pub repository: R,
    pub resolver: ResolverRegistry,
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
        let task = self.tasks.create(id, source, destination)?.clone();
        self.repository.insert(StoredTask::from(task.clone()))?;
        Ok(task)
    }

    pub fn queue_task(
        &mut self,
        id: &TaskId,
        priority: Priority,
    ) -> Result<(), CoreError> {
        self.scheduler.enqueue(&mut self.tasks, id, priority)?;
        self.persist_task(id)
    }

    pub fn start_next(&mut self) -> Result<Option<TaskId>, CoreError> {
        let id = self.scheduler.start_next(&mut self.tasks)?;
        if let Some(task_id) = &id {
            self.persist_task(task_id)?;
        }
        Ok(id)
    }

    pub fn pause_task(&mut self, id: &TaskId) -> Result<(), CoreError> {
        self.scheduler.pause(&mut self.tasks, id)?;
        self.persist_task(id)
    }

    pub fn resume_task(&mut self, id: &TaskId) -> Result<bool, CoreError> {
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
        let task = self.tasks.remove(id)?;
        self.repository.remove(id)?;
        Ok(task)
    }

    pub fn persist_task(&mut self, id: &TaskId) -> Result<(), CoreError> {
        let task = self.tasks.get(id)
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
        SchedulerConfig { max_concurrent_tasks: 2, ..SchedulerConfig::default() }
    }

    fn task_source() -> (DownloadSource, Destination) {
        (DownloadSource::new("https://example.com/file"), Destination::new("/tmp/file"))
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
        core.update_progress(&id, Progress::new(128, Some(1024))).unwrap();

        let stored = core.repository.get(&id).unwrap().unwrap();
        assert_eq!(stored.state, TaskState::Queued);
        assert_eq!(stored.progress, Progress::new(128, Some(1024)));
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
        let mut core = Core::with_repository(config(), SqliteRepository::open_in_memory().unwrap()).unwrap();
        let (source, destination) = task_source();
        let id = TaskId::from("sqlite-task");
        core.create_task(id.clone(), source, destination).unwrap();
        core.queue_task(&id, Priority::NORMAL).unwrap();
        core.start_next().unwrap();
        core.update_progress(&id, Progress::new(64, Some(100))).unwrap();

        let stored = core.repository.get(&id).unwrap().unwrap();
        assert_eq!(stored.state, TaskState::Downloading);
        assert_eq!(stored.progress.downloaded_bytes, 64);
    }
}
