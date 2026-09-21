//! Queue and concurrency policy for Nexum task scheduling.

use nexum_domain::TaskId;
use nexum_task::{TaskService, TaskState, TaskServiceError};
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Priority(u8);

impl Priority {
    pub const LOW: Self = Self(0);
    pub const NORMAL: Self = Self(50);
    pub const HIGH: Self = Self(100);

    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerConfig {
    pub max_concurrent_tasks: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 3,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct QueueEntry {
    task_id: TaskId,
    priority: Priority,
    sequence: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SchedulerError {
    InvalidConcurrencyLimit,
    Task(TaskServiceError),
}

impl From<TaskServiceError> for SchedulerError {
    fn from(value: TaskServiceError) -> Self {
        Self::Task(value)
    }
}

/// Deterministic in-memory scheduler.
///
/// The scheduler decides which queued tasks may start. It does not execute
/// network I/O and therefore remains independent of a concrete download engine.
#[derive(Debug)]
pub struct Scheduler {
    config: SchedulerConfig,
    queue: VecDeque<QueueEntry>,
    active_tasks: usize,
    next_sequence: u64,
}

impl Scheduler {
    pub fn new(config: SchedulerConfig) -> Result<Self, SchedulerError> {
        if config.max_concurrent_tasks == 0 {
            return Err(SchedulerError::InvalidConcurrencyLimit);
        }

        Ok(Self {
            config,
            queue: VecDeque::new(),
            active_tasks: 0,
            next_sequence: 0,
        })
    }

    pub fn enqueue(
        &mut self,
        task_service: &mut TaskService,
        task_id: &TaskId,
        priority: Priority,
    ) -> Result<(), SchedulerError> {
        task_service.transition(task_id, TaskState::Queued)?;
        self.queue.push_back(QueueEntry {
            task_id: task_id.clone(),
            priority,
            sequence: self.next_sequence,
        });
        self.next_sequence += 1;
        self.sort_queue();
        Ok(())
    }

    pub fn start_next(
        &mut self,
        task_service: &mut TaskService,
    ) -> Result<Option<TaskId>, SchedulerError> {
        if self.active_tasks >= self.config.max_concurrent_tasks {
            return Ok(None);
        }

        let Some(entry) = self.queue.pop_front() else {
            return Ok(None);
        };

        task_service.transition(&entry.task_id, TaskState::Downloading)?;
        self.active_tasks += 1;

        Ok(Some(entry.task_id))
    }

    pub fn mark_finished(
        &mut self,
        task_service: &mut TaskService,
        task_id: &TaskId,
        state: TaskState,
    ) -> Result<(), SchedulerError> {
        if !matches!(state, TaskState::Completed | TaskState::Failed) {
            return Err(SchedulerError::Task(TaskServiceError::InvalidTransition(
                nexum_task::InvalidTransition {
                    from: TaskState::Downloading,
                    to: state,
                },
            )));
        }

        task_service.transition(task_id, state)?;
        self.active_tasks = self.active_tasks.saturating_sub(1);
        Ok(())
    }

    pub fn queued_len(&self) -> usize {
        self.queue.len()
    }

    pub fn active_len(&self) -> usize {
        self.active_tasks
    }

    fn sort_queue(&mut self) {
        let mut entries: Vec<_> = self.queue.drain(..).collect();
        entries.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.sequence.cmp(&b.sequence))
        });
        self.queue = entries.into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexum_domain::{Destination, DownloadSource};

    fn service_with_tasks(count: usize) -> (TaskService, Vec<TaskId>) {
        let mut service = TaskService::new();
        let mut ids = Vec::new();

        for index in 0..count {
            let id = TaskId::from(format!("task-{index}"));
            service
                .create(
                    id.clone(),
                    DownloadSource::new(format!("https://example.com/{index}")),
                    Destination::new(format!("/tmp/{index}")),
                )
                .unwrap();
            ids.push(id);
        }

        (service, ids)
    }

    #[test]
    fn rejects_zero_concurrency() {
        let result = Scheduler::new(SchedulerConfig {
            max_concurrent_tasks: 0,
        });

        assert_eq!(result, Err(SchedulerError::InvalidConcurrencyLimit));
    }

    #[test]
    fn respects_concurrency_limit() {
        let (mut service, ids) = service_with_tasks(2);
        let mut scheduler = Scheduler::new(SchedulerConfig {
            max_concurrent_tasks: 1,
        })
        .unwrap();

        scheduler.enqueue(&mut service, &ids[0], Priority::NORMAL).unwrap();
        scheduler.enqueue(&mut service, &ids[1], Priority::NORMAL).unwrap();

        assert_eq!(scheduler.start_next(&mut service).unwrap(), Some(ids[0].clone()));
        assert_eq!(scheduler.start_next(&mut service).unwrap(), None);
        assert_eq!(scheduler.active_len(), 1);
        assert_eq!(scheduler.queued_len(), 1);
    }

    #[test]
    fn higher_priority_starts_first() {
        let (mut service, ids) = service_with_tasks(2);
        let mut scheduler = Scheduler::new(SchedulerConfig::default()).unwrap();

        scheduler.enqueue(&mut service, &ids[0], Priority::LOW).unwrap();
        scheduler.enqueue(&mut service, &ids[1], Priority::HIGH).unwrap();

        assert_eq!(scheduler.start_next(&mut service).unwrap(), Some(ids[1].clone()));
    }

    #[test]
    fn equal_priority_preserves_fifo_order() {
        let (mut service, ids) = service_with_tasks(2);
        let mut scheduler = Scheduler::new(SchedulerConfig::default()).unwrap();

        scheduler.enqueue(&mut service, &ids[0], Priority::NORMAL).unwrap();
        scheduler.enqueue(&mut service, &ids[1], Priority::NORMAL).unwrap();

        assert_eq!(scheduler.start_next(&mut service).unwrap(), Some(ids[0].clone()));
        assert_eq!(scheduler.start_next(&mut service).unwrap(), Some(ids[1].clone()));
    }

    #[test]
    fn finished_task_frees_a_slot() {
        let (mut service, ids) = service_with_tasks(2);
        let mut scheduler = Scheduler::new(SchedulerConfig {
            max_concurrent_tasks: 1,
        })
        .unwrap();

        scheduler.enqueue(&mut service, &ids[0], Priority::NORMAL).unwrap();
        scheduler.enqueue(&mut service, &ids[1], Priority::NORMAL).unwrap();
        scheduler.start_next(&mut service).unwrap();

        scheduler
            .mark_finished(&mut service, &ids[0], TaskState::Completed)
            .unwrap();

        assert_eq!(scheduler.active_len(), 0);
        assert_eq!(scheduler.start_next(&mut service).unwrap(), Some(ids[1].clone()));
    }
}
