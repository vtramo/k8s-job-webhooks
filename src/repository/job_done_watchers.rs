use std::sync::{Arc, OnceLock};
use async_trait::async_trait;
use moka::sync::Cache;

use crate::models::JobDoneWatcher;
use crate::repository::CrudRepository;

pub trait JobDoneWatcherRepository: CrudRepository<Entity = JobDoneWatcher> {}

pub struct InMemoryJobDoneWatcherRepository {
    job_done_watcher_by_id: Cache<String, JobDoneWatcher>,
}

impl InMemoryJobDoneWatcherRepository {
    pub fn new() -> Self {
        Self {
            job_done_watcher_by_id: Cache::new(15),
        }
    }
}

impl JobDoneWatcherRepository for InMemoryJobDoneWatcherRepository {}

#[async_trait]
impl CrudRepository for InMemoryJobDoneWatcherRepository {
    type Entity = JobDoneWatcher;

    async fn find_all(&self) -> Vec<JobDoneWatcher> {
        self.job_done_watcher_by_id.iter()
            .map(|(_, webhook)| webhook)
            .collect()
    }

    async fn find_by_id(&self, id: &str) -> Option<JobDoneWatcher> {
        self.job_done_watcher_by_id.get(id)
    }

    async fn save(&self, webhook: JobDoneWatcher) {
        self.job_done_watcher_by_id.insert(webhook.id.clone(), webhook);
    }
}

pub static JOB_DONE_WATCHER_REPOSITORY: OnceLock<Arc<dyn JobDoneWatcherRepository>> = OnceLock::new();

pub fn set_job_done_watcher_repository(job_done_watcher_repository: impl JobDoneWatcherRepository + 'static) {
    if let Err(_) = JOB_DONE_WATCHER_REPOSITORY.set(Arc::new(job_done_watcher_repository)) {
        panic!("You can't set Webhook Repository twice!");
    }
}

pub fn get_job_done_watcher_repository() -> Arc<dyn JobDoneWatcherRepository> {
    JOB_DONE_WATCHER_REPOSITORY.get().expect("Should be set!").clone()
}