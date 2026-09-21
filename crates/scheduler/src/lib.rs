//! Queue, retry, pause/resume, and concurrency policy for Nexum task scheduling.

use nexum_domain::TaskId;
use nexum_task::{InvalidTransition, TaskService, TaskState, TaskServiceError};
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Priority(u8);

impl Priority {
    pub const LOW: Self = Self(0);
    pub const NORMAL: Self = Self(50);
    pub const HIGH: Self = Self(100);

    pub const fn new(value: u8) -> Self { Self(value) }
    pub const fn value(self) -> u8 { self.0 }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetryPolicy {
    pub max_retries: u32,
}

impl Default for RetryPolicy {
    fn default() -> Self { Self { max_retries: 3 } }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerConfig {
    pub max_concurrent_tasks: usize,
    pub retry_policy: RetryPolicy,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self { max_concurrent_tasks: 3, retry_policy: RetryPolicy::default() }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct QueueEntry {
    task_id: TaskId,
    priority: Priority,
    sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SchedulerEvent {
    Enqueued { task_id: TaskId, priority: Priority },
    Started { task_id: TaskId },
    Paused { task_id: TaskId },
    Resumed { task_id: TaskId },
    Completed { task_id: TaskId },
    Failed { task_id: TaskId },
    Retrying { task_id: TaskId, attempt: u32 },
}

#[derive(Debug, Eq, PartialEq)]
pub enum SchedulerError {
    InvalidConcurrencyLimit,
    Task(TaskServiceError),
}

impl From<TaskServiceError> for SchedulerError {
    fn from(value: TaskServiceError) -> Self { Self::Task(value) }
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
    retry_counts: HashMap<TaskId, u32>,
    events: Vec<SchedulerEvent>,
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
            retry_counts: HashMap::new(),
            events: Vec::new(),
        })
    }

    pub fn enqueue(
        &mut self,
        task_service: &mut TaskService,
        task_id: &TaskId,
        priority: Priority,
    ) -> Result<(), SchedulerError> {
        task_service.transition(task_id, TaskState::Queued)?;
        self.push_queue(task_id, priority);
        self.events.push(SchedulerEvent::Enqueued { task_id: task_id.clone(), priority });
        Ok(())
    }

    pub fn start_next(
        &mut self,
        task_service: &mut TaskService,
    ) -> Result<Option<TaskId>, SchedulerError> {
        if self.active_tasks >= self.config.max_concurrent_tasks {
            return Ok(None);
        }
        let Some(entry) = self.queue.pop_front() else { return Ok(None); };
        task_service.transition(&entry.task_id, TaskState::Downloading)?;
        self.active_tasks += 1;
        self.events.push(SchedulerEvent::Started { task_id: entry.task_id.clone() });
        Ok(Some(entry.task_id))
    }

    pub fn pause(
        &mut self,
        task_service: &mut TaskService,
        task_id: &TaskId,
    ) -> Result<(), SchedulerError> {
        task_service.transition(task_id, TaskState::Paused)?;
        self.active_tasks = self.active_tasks.saturating_sub(1);
        self.events.push(SchedulerEvent::Paused { task_id: task_id.clone() });
        Ok(())
    }

    pub fn resume(
        &mut self,
        task_service: &mut TaskService,
        task_id: &TaskId,
    ) -> Result<bool, SchedulerError> {
        if self.active_tasks >= self.config.max_concurrent_tasks {
            return Ok(false);
        }
        task_service.transition(task_id, TaskState::Downloading)?;
        self.active_tasks += 1;
        self.events.push(SchedulerEvent::Resumed { task_id: task_id.clone() });
        Ok(true)
    }

    pub fn mark_finished(
        &mut self,
        task_service: &mut TaskService,
        task_id: &TaskId,
        state: TaskState,
    ) -> Result<(), SchedulerError> {
        if !matches!(state, TaskState::Completed | TaskState::Failed) {
            return Err(SchedulerError::Task(TaskServiceError::InvalidTransition(
                InvalidTransition { from: TaskState::Downloading, to: state },
            )));
        }

        task_service.transition(task_id, state)?;
        self.active_tasks = self.active_tasks.saturating_sub(1);

        match state {
            TaskState::Completed => {
                self.retry_counts.remove(task_id);
                self.events.push(SchedulerEvent::Completed { task_id: task_id.clone() });
            }
            TaskState::Failed => {
                let attempt = self.retry_counts.entry(task_id.clone()).or_insert(0);
                if *attempt < self.config.retry_policy.max_retries {
                    *attempt += 1;
                    let attempt_number = *attempt;
                    task_service.transition(task_id, TaskState::Retrying)?;
                    task_service.transition(task_id, TaskState::Queued)?;
                    self.push_queue(task_id, Priority::NORMAL);
                    self.events.push(SchedulerEvent::Retrying {
                        task_id: task_id.clone(),
                        attempt: attempt_number,
                    });
                } else {
                    self.retry_counts.remove(task_id);
                    self.events.push(SchedulerEvent::Failed { task_id: task_id.clone() });
                }
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    pub fn queued_len(&self) -> usize { self.queue.len() }
    pub fn active_len(&self) -> usize { self.active_tasks }

    pub fn drain_events(&mut self) -> Vec<SchedulerEvent> {
        std::mem::take(&mut self.events)
    }

    fn push_queue(&mut self, task_id: &TaskId, priority: Priority) {
        self.queue.push_back(QueueEntry {
            task_id: task_id.clone(),
            priority,
            sequence: self.next_sequence,
        });
        self.next_sequence += 1;
        self.sort_queue();
    }

    fn sort_queue(&mut self) {
        let mut entries: Vec<_> = self.queue.drain(..).collect();
        entries.sort_by(|a, b| {
            b.priority.cmp(&a.priority).then_with(|| a.sequence.cmp(&b.sequence))
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
            service.create(
                id.clone(),
                DownloadSource::new(format!("https://example.com/{index}")),
                Destination::new(format!("/tmp/{index}")),
            ).unwrap();
            ids.push(id);
        }
        (service, ids)
    }

    #[test]
    fn rejects_zero_concurrency() {
        let result = Scheduler::new(SchedulerConfig {
            max_concurrent_tasks: 0,
            ..SchedulerConfig::default()
        });
        assert_eq!(result, Err(SchedulerError::InvalidConcurrencyLimit));
    }

    #[test]
    fn respects_concurrency_limit() {
        let (mut service, ids) = service_with_tasks(2);
        let mut scheduler = Scheduler::new(SchedulerConfig {
            max_concurrent_tasks: 1,
            ..SchedulerConfig::default()
        }).unwrap();
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
            ..SchedulerConfig::default()
        }).unwrap();
        scheduler.enqueue(&mut service, &ids[0], Priority::NORMAL).unwrap();
        scheduler.enqueue(&mut service, &ids[1], Priority::NORMAL).unwrap();
        scheduler.start_next(&mut service).unwrap();
        scheduler.mark_finished(&mut service, &ids[0], TaskState::Completed).unwrap();
        assert_eq!(scheduler.active_len(), 0);
        assert_eq!(scheduler.start_next(&mut service).unwrap(), Some(ids[1].clone()));
    }

    #[test]
    fn pause_frees_slot_and_resume_uses_it() {
        let (mut service, ids) = service_with_tasks(2);
        let mut scheduler = Scheduler::new(SchedulerConfig {
            max_concurrent_tasks: 1,
            ..SchedulerConfig::default()
        }).unwrap();
        scheduler.enqueue(&mut service, &ids[0], Priority::NORMAL).unwrap();
        scheduler.enqueue(&mut service, &ids[1], Priority::NORMAL).unwrap();
        scheduler.start_next(&mut service).unwrap();
        scheduler.pause(&mut service, &ids[0]).unwrap();
        assert_eq!(scheduler.active_len(), 0);
        assert_eq!(service.get(&ids[0]).unwrap().state, TaskState::Paused);
        assert!(scheduler.resume(&mut service, &ids[0]).unwrap());
        assert_eq!(scheduler.active_len(), 1);
    }

    #[test]
    fn resume_waits_when_no_slot_is_available() {
        let (mut service, ids) = service_with_tasks(2);
        let mut scheduler = Scheduler::new(SchedulerConfig {
            max_concurrent_tasks: 1,
            ..SchedulerConfig::default()
        }).unwrap();
        scheduler.enqueue(&mut service, &ids[0], Priority::NORMAL).unwrap();
        scheduler.enqueue(&mut service, &ids[1], Priority::NORMAL).unwrap();
        scheduler.start_next(&mut service).unwrap();
        scheduler.pause(&mut service, &ids[0]).unwrap();
        scheduler.start_next(&mut service).unwrap();
        assert!(!scheduler.resume(&mut service, &ids[0]).unwrap());
        assert_eq!(service.get(&ids[0]).unwrap().state, TaskState::Paused);
    }

    #[test]
    fn failed_task_is_requeued_until_retry_budget_is_exhausted() {
        let (mut service, ids) = service_with_tasks(1);
        let mut scheduler = Scheduler::new(SchedulerConfig {
            max_concurrent_tasks: 1,
            retry_policy: RetryPolicy { max_retries: 2 },
        }).unwrap();
        scheduler.enqueue(&mut service, &ids[0], Priority::NORMAL).unwrap();
        scheduler.start_next(&mut service).unwrap();
        scheduler.mark_finished(&mut service, &ids[0], TaskState::Failed).unwrap();
        assert_eq!(service.get(&ids[0]).unwrap().state, TaskState::Queued);
        assert_eq!(scheduler.queued_len(), 1);
        assert_eq!(scheduler.active_len(), 0);
        scheduler.start_next(&mut service).unwrap();
        scheduler.mark_finished(&mut service, &ids[0], TaskState::Failed).unwrap();
        scheduler.start_next(&mut service).unwrap();
        scheduler.mark_finished(&mut service, &ids[0], TaskState::Failed).unwrap();
        assert_eq!(service.get(&ids[0]).unwrap().state, TaskState::Failed);
        assert_eq!(scheduler.queued_len(), 0);
        assert_eq!(scheduler.active_len(), 0);
    }

    #[test]
    fn retry_events_are_emitted() {
        let (mut service, ids) = service_with_tasks(1);
        let mut scheduler = Scheduler::new(SchedulerConfig {
            max_concurrent_tasks: 1,
            retry_policy: RetryPolicy { max_retries: 1 },
        }).unwrap();
        scheduler.enqueue(&mut service, &ids[0], Priority::HIGH).unwrap();
        scheduler.start_next(&mut service).unwrap();
        scheduler.mark_finished(&mut service, &ids[0], TaskState::Failed).unwrap();
        assert_eq!(scheduler.drain_events(), vec![
            SchedulerEvent::Enqueued { task_id: ids[0].clone(), priority: Priority::HIGH },
            SchedulerEvent::Started { task_id: ids[0].clone() },
            SchedulerEvent::Retrying { task_id: ids[0].clone(), attempt: 1 },
        ]);
    }
}
