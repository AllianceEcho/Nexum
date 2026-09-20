//! Download task model, lifecycle state machine, events, and in-memory service.

use nexum_domain::{Destination, DownloadSource, Progress, TaskId};
use std::{collections::HashMap, fmt};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskState {
    Created,
    Queued,
    Downloading,
    Paused,
    Completed,
    Failed,
    Retrying,
}

impl TaskState {
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Created, Self::Queued)
                | (Self::Queued, Self::Downloading)
                | (Self::Downloading, Self::Paused)
                | (Self::Downloading, Self::Completed)
                | (Self::Downloading, Self::Failed)
                | (Self::Paused, Self::Downloading)
                | (Self::Failed, Self::Retrying)
                | (Self::Retrying, Self::Queued)
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidTransition {
    pub from: TaskState,
    pub to: TaskState,
}

impl fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid task transition: {:?} -> {:?}", self.from, self.to)
    }
}

impl std::error::Error for InvalidTransition {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DownloadTask {
    pub id: TaskId,
    pub source: DownloadSource,
    pub destination: Destination,
    pub state: TaskState,
    pub progress: Progress,
}

impl DownloadTask {
    pub fn new(
        id: impl Into<TaskId>,
        source: DownloadSource,
        destination: Destination,
    ) -> Self {
        Self {
            id: id.into(),
            source,
            destination,
            state: TaskState::Created,
            progress: Progress::default(),
        }
    }

    pub fn transition_to(&mut self, next: TaskState) -> Result<(), InvalidTransition> {
        if !self.state.can_transition_to(next) {
            return Err(InvalidTransition {
                from: self.state,
                to: next,
            });
        }

        self.state = next;
        Ok(())
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self.state, TaskState::Completed)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TaskEvent {
    Created {
        task_id: TaskId,
    },
    StateChanged {
        task_id: TaskId,
        from: TaskState,
        to: TaskState,
    },
    ProgressChanged {
        task_id: TaskId,
        progress: Progress,
    },
    Removed {
        task_id: TaskId,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum TaskServiceError {
    AlreadyExists(TaskId),
    NotFound(TaskId),
    InvalidTransition(InvalidTransition),
}

impl fmt::Display for TaskServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyExists(id) => write!(f, "task already exists: {id}"),
            Self::NotFound(id) => write!(f, "task not found: {id}"),
            Self::InvalidTransition(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for TaskServiceError {}

impl From<InvalidTransition> for TaskServiceError {
    fn from(value: InvalidTransition) -> Self {
        Self::InvalidTransition(value)
    }
}

/// In-memory task service used by the Core during the initial implementation.
#[derive(Default)]
pub struct TaskService {
    tasks: HashMap<TaskId, DownloadTask>,
    events: Vec<TaskEvent>,
}

impl TaskService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(
        &mut self,
        id: impl Into<TaskId>,
        source: DownloadSource,
        destination: Destination,
    ) -> Result<&DownloadTask, TaskServiceError> {
        let id = id.into();

        if self.tasks.contains_key(&id) {
            return Err(TaskServiceError::AlreadyExists(id));
        }

        self.tasks
            .insert(id.clone(), DownloadTask::new(id.clone(), source, destination));
        self.events.push(TaskEvent::Created { task_id: id.clone() });

        Ok(self.tasks.get(&id).expect("task was inserted"))
    }

    pub fn get(&self, id: &TaskId) -> Option<&DownloadTask> {
        self.tasks.get(id)
    }

    pub fn list(&self) -> impl Iterator<Item = &DownloadTask> {
        self.tasks.values()
    }

    pub fn transition(
        &mut self,
        id: &TaskId,
        next: TaskState,
    ) -> Result<&DownloadTask, TaskServiceError> {
        let task = self
            .tasks
            .get_mut(id)
            .ok_or_else(|| TaskServiceError::NotFound(id.clone()))?;

        let from = task.state;
        task.transition_to(next)?;
        self.events.push(TaskEvent::StateChanged {
            task_id: id.clone(),
            from,
            to: next,
        });

        Ok(self.tasks.get(id).expect("task still exists"))
    }

    pub fn update_progress(
        &mut self,
        id: &TaskId,
        progress: Progress,
    ) -> Result<&DownloadTask, TaskServiceError> {
        let task = self
            .tasks
            .get_mut(id)
            .ok_or_else(|| TaskServiceError::NotFound(id.clone()))?;

        task.progress = progress.clone();
        self.events.push(TaskEvent::ProgressChanged {
            task_id: id.clone(),
            progress,
        });

        Ok(self.tasks.get(id).expect("task still exists"))
    }

    pub fn remove(&mut self, id: &TaskId) -> Result<DownloadTask, TaskServiceError> {
        let task = self
            .tasks
            .remove(id)
            .ok_or_else(|| TaskServiceError::NotFound(id.clone()))?;

        self.events.push(TaskEvent::Removed {
            task_id: id.clone(),
        });

        Ok(task)
    }

    pub fn drain_events(&mut self) -> Vec<TaskEvent> {
        std::mem::take(&mut self.events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task() -> DownloadTask {
        DownloadTask::new(
            "task-1",
            DownloadSource::new("https://example.com/file"),
            Destination::new("/tmp/file"),
        )
    }

    #[test]
    fn initial_state_is_created() {
        assert_eq!(task().state, TaskState::Created);
    }

    #[test]
    fn valid_lifecycle_can_reach_completed() {
        let mut task = task();

        for state in [
            TaskState::Queued,
            TaskState::Downloading,
            TaskState::Completed,
        ] {
            task.transition_to(state).unwrap();
        }

        assert!(task.is_terminal());
    }

    #[test]
    fn paused_task_can_resume() {
        let mut task = task();
        task.transition_to(TaskState::Queued).unwrap();
        task.transition_to(TaskState::Downloading).unwrap();
        task.transition_to(TaskState::Paused).unwrap();
        task.transition_to(TaskState::Downloading).unwrap();

        assert_eq!(task.state, TaskState::Downloading);
    }

    #[test]
    fn failed_task_can_retry() {
        let mut task = task();
        task.transition_to(TaskState::Queued).unwrap();
        task.transition_to(TaskState::Downloading).unwrap();
        task.transition_to(TaskState::Failed).unwrap();
        task.transition_to(TaskState::Retrying).unwrap();
        task.transition_to(TaskState::Queued).unwrap();

        assert_eq!(task.state, TaskState::Queued);
    }

    #[test]
    fn invalid_transition_is_rejected() {
        let mut task = task();

        let error = task.transition_to(TaskState::Completed).unwrap_err();

        assert_eq!(
            error,
            InvalidTransition {
                from: TaskState::Created,
                to: TaskState::Completed
            }
        );
        assert_eq!(task.state, TaskState::Created);
    }

    #[test]
    fn completed_is_terminal() {
        let mut task = task();
        task.transition_to(TaskState::Queued).unwrap();
        task.transition_to(TaskState::Downloading).unwrap();
        task.transition_to(TaskState::Completed).unwrap();

        assert!(!task.state.can_transition_to(TaskState::Downloading));
        assert!(task.is_terminal());
    }

    #[test]
    fn service_emits_events() {
        let mut service = TaskService::new();
        let id = TaskId::from("task-1");

        service
            .create(
                id.clone(),
                DownloadSource::new("https://example.com/file"),
                Destination::new("/tmp/file"),
            )
            .unwrap();
        service.transition(&id, TaskState::Queued).unwrap();
        service.update_progress(&id, Progress::new(128, Some(1024))).unwrap();

        assert_eq!(
            service.drain_events(),
            vec![
                TaskEvent::Created { task_id: id.clone() },
                TaskEvent::StateChanged {
                    task_id: id.clone(),
                    from: TaskState::Created,
                    to: TaskState::Queued,
                },
                TaskEvent::ProgressChanged {
                    task_id: id,
                    progress: Progress::new(128, Some(1024)),
                },
            ]
        );
    }

    #[test]
    fn service_rejects_duplicate_tasks() {
        let mut service = TaskService::new();
        service
            .create(
                "task-1",
                DownloadSource::new("https://example.com/file"),
                Destination::new("/tmp/file"),
            )
            .unwrap();

        let error = service
            .create(
                "task-1",
                DownloadSource::new("https://example.com/other"),
                Destination::new("/tmp/other"),
            )
            .unwrap_err();

        assert_eq!(error, TaskServiceError::AlreadyExists(TaskId::from("task-1")));
    }
}
