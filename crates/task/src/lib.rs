//! Download task model and lifecycle state machine.

use nexum_domain::{Destination, DownloadSource, Progress, TaskId};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskState { Created, Queued, Downloading, Paused, Completed, Failed, Retrying }

impl TaskState {
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!((self, next),
            (Self::Created, Self::Queued) |
            (Self::Queued, Self::Downloading) |
            (Self::Downloading, Self::Paused) |
            (Self::Downloading, Self::Completed) |
            (Self::Downloading, Self::Failed) |
            (Self::Paused, Self::Downloading) |
            (Self::Failed, Self::Retrying) |
            (Self::Retrying, Self::Queued))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidTransition { pub from: TaskState, pub to: TaskState }

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
    pub fn new(id: impl Into<TaskId>, source: DownloadSource, destination: Destination) -> Self {
        Self { id: id.into(), source, destination, state: TaskState::Created, progress: Progress::default() }
    }

    pub fn transition_to(&mut self, next: TaskState) -> Result<(), InvalidTransition> {
        if !self.state.can_transition_to(next) {
            return Err(InvalidTransition { from: self.state, to: next });
        }
        self.state = next;
        Ok(())
    }

    pub fn is_terminal(&self) -> bool { matches!(self.state, TaskState::Completed) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task() -> DownloadTask {
        DownloadTask::new("task-1", DownloadSource::new("https://example.com/file"), Destination::new("/tmp/file"))
    }

    #[test]
    fn initial_state_is_created() { assert_eq!(task().state, TaskState::Created); }

    #[test]
    fn valid_lifecycle_can_reach_completed() {
        let mut task = task();
        for state in [TaskState::Queued, TaskState::Downloading, TaskState::Completed] { task.transition_to(state).unwrap(); }
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
        assert_eq!(error, InvalidTransition { from: TaskState::Created, to: TaskState::Completed });
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
}
