use std::sync::{Arc, OnceLock};
use async_trait::async_trait;
use moka::sync::Cache;

use crate::models::JobDoneWatcher;
use crate::repository::CrudRepository;

#[async_trait]
pub trait JobDoneWatcherRepository: CrudRepository<Entity = JobDoneWatcher> {
    async fn find_all_by_job_name(&self, job_name: &str) -> Vec<JobDoneWatcher>;
}

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

#[async_trait]
impl JobDoneWatcherRepository for InMemoryJobDoneWatcherRepository {
    async fn find_all_by_job_name(&self, job_name: &str) -> Vec<JobDoneWatcher> {
        self.job_done_watcher_by_id.iter()
            .filter(|(job_done_watcher_id, job_done_watcher)| {
                job_done_watcher.job_name == job_name
            }).map(|(_, job_done_watcher)| job_done_watcher)
            .collect()
    }
}

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