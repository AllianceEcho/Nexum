//! Persistence boundaries for Nexum task state.

use nexum_domain::{Destination, DownloadSource, Progress, TaskId};
use nexum_task::{DownloadTask, TaskState};
use std::{collections::HashMap, fmt};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageError {
    NotFound(TaskId),
    AlreadyExists(TaskId),
    Other(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "task not found: {id}"),
            Self::AlreadyExists(id) => write!(f, "task already exists: {id}"),
            Self::Other(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for StorageError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredTask {
    pub id: TaskId,
    pub source: DownloadSource,
    pub destination: Destination,
    pub state: TaskState,
    pub progress: Progress,
}

impl From<DownloadTask> for StoredTask {
    fn from(task: DownloadTask) -> Self {
        Self {
            id: task.id,
            source: task.source,
            destination: task.destination,
            state: task.state,
            progress: task.progress,
        }
    }
}

impl StoredTask {
    pub fn into_task(self) -> DownloadTask {
        DownloadTask {
            id: self.id,
            source: self.source,
            destination: self.destination,
            state: self.state,
            progress: self.progress,
        }
    }
}

pub trait TaskRepository {
    fn insert(&mut self, task: StoredTask) -> Result<(), StorageError>;
    fn get(&self, id: &TaskId) -> Result<Option<StoredTask>, StorageError>;
    fn list(&self) -> Result<Vec<StoredTask>, StorageError>;
    fn update(&mut self, task: StoredTask) -> Result<(), StorageError>;
    fn remove(&mut self, id: &TaskId) -> Result<Option<StoredTask>, StorageError>;
}

#[derive(Default, Debug)]
pub struct InMemoryRepository {
    tasks: HashMap<TaskId, StoredTask>,
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TaskRepository for InMemoryRepository {
    fn insert(&mut self, task: StoredTask) -> Result<(), StorageError> {
        if self.tasks.contains_key(&task.id) {
            return Err(StorageError::AlreadyExists(task.id));
        }
        self.tasks.insert(task.id.clone(), task);
        Ok(())
    }

    fn get(&self, id: &TaskId) -> Result<Option<StoredTask>, StorageError> {
        Ok(self.tasks.get(id).cloned())
    }

    fn list(&self) -> Result<Vec<StoredTask>, StorageError> {
        Ok(self.tasks.values().cloned().collect())
    }

    fn update(&mut self, task: StoredTask) -> Result<(), StorageError> {
        if !self.tasks.contains_key(&task.id) {
            return Err(StorageError::NotFound(task.id));
        }
        self.tasks.insert(task.id.clone(), task);
        Ok(())
    }

    fn remove(&mut self, id: &TaskId) -> Result<Option<StoredTask>, StorageError> {
        Ok(self.tasks.remove(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexum_domain::{Destination, DownloadSource};

    fn stored_task(id: &str) -> StoredTask {
        DownloadTask::new(
            id,
            DownloadSource::new(format!("https://example.com/{id}")),
            Destination::new(format!("/tmp/{id}")),
        ).into()
    }

    #[test]
    fn insert_and_get_round_trip() {
        let mut repo = InMemoryRepository::new();
        let task = stored_task("task-1");
        repo.insert(task.clone()).unwrap();
        assert_eq!(repo.get(&TaskId::from("task-1")).unwrap(), Some(task));
    }

    #[test]
    fn duplicate_insert_is_rejected() {
        let mut repo = InMemoryRepository::new();
        repo.insert(stored_task("task-1")).unwrap();
        assert_eq!(
            repo.insert(stored_task("task-1")).unwrap_err(),
            StorageError::AlreadyExists(TaskId::from("task-1"))
        );
    }

    #[test]
    fn update_requires_existing_task() {
        let mut repo = InMemoryRepository::new();
        let task = stored_task("task-1");
        assert_eq!(
            repo.update(task).unwrap_err(),
            StorageError::NotFound(TaskId::from("task-1"))
        );
    }

    #[test]
    fn remove_returns_task() {
        let mut repo = InMemoryRepository::new();
        let task = stored_task("task-1");
        repo.insert(task.clone()).unwrap();
        assert_eq!(
            repo.remove(&TaskId::from("task-1")).unwrap(),
            Some(task)
        );
        assert_eq!(repo.get(&TaskId::from("task-1")).unwrap(), None);
    }

    #[test]
    fn task_conversion_round_trips() {
        let task = stored_task("task-1").into_task();
        assert_eq!(task.id, TaskId::from("task-1"));
        assert_eq!(task.state, TaskState::Created);
    }
}
