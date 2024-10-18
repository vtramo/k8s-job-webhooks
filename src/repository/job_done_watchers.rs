use std::future::Future;
use std::sync::{Arc, OnceLock};

use async_rwlock::RwLock;
use async_trait::async_trait;
use futures_util::stream;
use futures_util::StreamExt;
use moka::sync::Cache;

use crate::models::{JobDoneWatcher, JobDoneWatcherStatus};
use crate::repository::{AsyncLockGuard, CrudRepository};

#[async_trait]
pub trait JobDoneWatcherRepository: CrudRepository<Entity = JobDoneWatcher> + AsyncLockGuard<JobDoneWatcher> {
    async fn find_all_by_job_name_and_status(&self, job_name: &str, status: JobDoneWatcherStatus) -> Vec<JobDoneWatcher>;
}

pub struct InMemoryJobDoneWatcherRepository {
    job_done_watcher_by_id: Cache<String, Arc<RwLock<JobDoneWatcher>>>,
}

impl InMemoryJobDoneWatcherRepository {
    pub fn new() -> Self {
        Self {
            job_done_watcher_by_id: Cache::new(15),
        }
    }
}

#[async_trait::async_trait]
impl AsyncLockGuard<JobDoneWatcher> for InMemoryJobDoneWatcherRepository {
    async fn lock(&self, id: &str, critical_section: Box<dyn FnOnce(JobDoneWatcher) -> Box<dyn Future<Output=()> + Send> + Send>) {
        let job_done_watcher = self.job_done_watcher_by_id.get(id).unwrap();
        let job_done_watcher = job_done_watcher.write().await;
        Box::into_pin(critical_section(job_done_watcher.clone())).await;
    }
}

#[async_trait]
impl JobDoneWatcherRepository for InMemoryJobDoneWatcherRepository {
    async fn find_all_by_job_name_and_status(&self, job_name: &str, status: JobDoneWatcherStatus) -> Vec<JobDoneWatcher> {
        stream::iter(self.job_done_watcher_by_id.iter())
            .filter_map(|(_, job_done_watcher): (_, Arc<RwLock<JobDoneWatcher>>)| {
                let job_done_watcher = Arc::clone(&job_done_watcher);
                async move {
                    let job_done_watcher = job_done_watcher.read().await;
                    if &job_done_watcher.job_name == job_name && job_done_watcher.status == status {
                        Some(job_done_watcher.clone())
                    } else {
                        None
                    }
                }
            })
            .collect::<Vec<_>>()
            .await
    }
}

#[async_trait]
impl CrudRepository for InMemoryJobDoneWatcherRepository {
    type Entity = JobDoneWatcher;

    async fn find_all(&self) -> Vec<JobDoneWatcher> {
        stream::iter(self.job_done_watcher_by_id.iter())
            .then(|(_, job_done_watcher)| {
                let job_done_watcher = Arc::clone(&job_done_watcher);
                async move { job_done_watcher.read().await.clone() }
            })
            .collect::<Vec<_>>()
            .await
    }

    async fn find_by_id(&self, id: &str) -> Option<JobDoneWatcher> {
        let job_done_watcher = Arc::clone(&self.job_done_watcher_by_id.get(id)?);
        let job_done_watcher = job_done_watcher.read().await;
        Some(job_done_watcher.clone())
    }

    async fn save(&self, job_done_watcher: JobDoneWatcher) {
        println!("Saving {:#?}", job_done_watcher);
        self.job_done_watcher_by_id.insert(job_done_watcher.id.clone(), Arc::new(RwLock::new(job_done_watcher)));
    }
}

pub static JOB_DONE_WATCHER_REPOSITORY: OnceLock<Arc<dyn JobDoneWatcherRepository>> = OnceLock::new();

pub fn set_job_done_watcher_repository(job_done_watcher_repository: impl JobDoneWatcherRepository + 'static) {
    if let Err(_) = JOB_DONE_WATCHER_REPOSITORY.set(Arc::new(job_done_watcher_repository)) {
        panic!("You can't set Webhook Repository twice!");
    }
}

pub fn get_job_done_watcher_repository() -> Arc<dyn JobDoneWatcherRepository> {
    Arc::clone(JOB_DONE_WATCHER_REPOSITORY.get().expect("Should be set!"))
}