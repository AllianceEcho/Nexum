//! Nexum Core orchestration layer.

pub use nexum_domain;
pub use nexum_scheduler;
pub use nexum_task;

use nexum_domain::{Destination, DownloadSource, TaskId};
use nexum_scheduler::{Priority, Scheduler, SchedulerConfig, SchedulerEvent};
use nexum_task::{DownloadTask, TaskService, TaskServiceError};

pub struct Core {
    pub tasks: TaskService,
    pub scheduler: Scheduler,
}

impl Core {
    pub fn new(scheduler_config: SchedulerConfig) -> Result<Self, nexum_scheduler::SchedulerError> {
        Ok(Self {
            tasks: TaskService::new(),
            scheduler: Scheduler::new(scheduler_config)?,
        })
    }

    pub fn create_task(
        &mut self,
        id: impl Into<TaskId>,
        source: DownloadSource,
        destination: Destination,
    ) -> Result<&DownloadTask, TaskServiceError> {
        self.tasks.create(id, source, destination)
    }

    pub fn queue_task(
        &mut self,
        id: &TaskId,
        priority: Priority,
    ) -> Result<(), nexum_scheduler::SchedulerError> {
        self.scheduler.enqueue(&mut self.tasks, id, priority)
    }

    pub fn start_next(
        &mut self,
    ) -> Result<Option<TaskId>, nexum_scheduler::SchedulerError> {
        self.scheduler.start_next(&mut self.tasks)
    }

    pub fn pause_task(
        &mut self,
        id: &TaskId,
    ) -> Result<(), nexum_scheduler::SchedulerError> {
        self.scheduler.pause(&mut self.tasks, id)
    }

    pub fn resume_task(
        &mut self,
        id: &TaskId,
    ) -> Result<bool, nexum_scheduler::SchedulerError> {
        self.scheduler.resume(&mut self.tasks, id)
    }

    pub fn finish_task(
        &mut self,
        id: &TaskId,
        state: nexum_task::TaskState,
    ) -> Result<(), nexum_scheduler::SchedulerError> {
        self.scheduler.mark_finished(&mut self.tasks, id, state)
    }

    pub fn drain_scheduler_events(&mut self) -> Vec<SchedulerEvent> {
        self.scheduler.drain_events()
    }
}
