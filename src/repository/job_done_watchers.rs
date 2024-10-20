use std::future::Future;
use std::sync::{Arc, OnceLock};

use anyhow::Context;
use async_rwlock::RwLock;
use async_trait::async_trait;
use futures_util::stream;
use futures_util::StreamExt;
use moka::sync::Cache;
use uuid::Uuid;

use crate::models::{JobDoneWatcher, JobDoneWatcherStatus};
use crate::models::entity::JobDoneWatcherEntity;
use crate::repository::{AsyncLockGuard, SqliteDatabase, SqlxAcquire};

#[async_trait]
pub trait JobDoneWatcherRepository: AsyncLockGuard<JobDoneWatcher> + Send + Sync {
    async fn find_all_watchers_by_job_name_and_status(
        &self,
        job_name: &str, // TODO: newtype pattern
        status: JobDoneWatcherStatus
    ) -> anyhow::Result<Vec<JobDoneWatcher>>;
    async fn find_all_watchers(&self) -> anyhow::Result<Vec<JobDoneWatcher>>;
    async fn find_watcher_by_id(&self, id: &Uuid) -> anyhow::Result<Option<JobDoneWatcher>>;
    async fn create_watcher(&self, job_done_watcher: &JobDoneWatcher) -> anyhow::Result<()>;
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
    async fn lock(&self, id: &Uuid, critical_section: Box<dyn FnOnce(JobDoneWatcher) -> Box<dyn Future<Output=anyhow::Result<()>> + Send> + Send>) -> anyhow::Result<()> {
        let job_done_watcher = self.job_done_watcher_by_id.get(&id.to_string()).unwrap();
        let job_done_watcher = job_done_watcher.write().await;
        Box::into_pin(critical_section(job_done_watcher.clone())).await
    }
}

#[async_trait]
impl JobDoneWatcherRepository for InMemoryJobDoneWatcherRepository {
    async fn find_all_watchers_by_job_name_and_status(
        &self,
        job_name: &str,
        status: JobDoneWatcherStatus
    ) -> anyhow::Result<Vec<JobDoneWatcher>> {
        Ok(stream::iter(self.job_done_watcher_by_id.iter())
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
            .await)
    }

    async fn find_all_watchers(&self) -> anyhow::Result<Vec<JobDoneWatcher>> {
        Ok(stream::iter(self.job_done_watcher_by_id.iter())
            .then(|(_, job_done_watcher)| {
                let job_done_watcher = Arc::clone(&job_done_watcher);
                async move { job_done_watcher.read().await.clone() }
            })
            .collect::<Vec<_>>()
            .await)
    }

    async fn find_watcher_by_id(&self, id: &Uuid) -> anyhow::Result<Option<JobDoneWatcher>> {
        if let Some(job_done_watcher) = self.job_done_watcher_by_id.get(&id.to_string()) {
            let job_done_watcher = Arc::clone(&job_done_watcher);
            let job_done_watcher = job_done_watcher.read().await;
            Ok(Some(job_done_watcher.clone()))
        } else {
            Ok(None)
        }
    }

    async fn create_watcher(&self, job_done_watcher: &JobDoneWatcher) -> anyhow::Result<()> {
        println!("Saving {:#?}", job_done_watcher);
        self.job_done_watcher_by_id.insert(job_done_watcher.id.to_string(), Arc::new(RwLock::new(job_done_watcher.clone())));
        Ok(())
    }
}

#[async_trait::async_trait]
impl AsyncLockGuard<JobDoneWatcher> for SqliteDatabase {
    async fn lock(&self, id: &Uuid, critical_section: Box<dyn FnOnce(JobDoneWatcher) -> Box<dyn Future<Output=anyhow::Result<()>> + Send> + Send>) -> anyhow::Result<()> {
        todo!()
    }
}


#[async_trait::async_trait]
impl JobDoneWatcherRepository for SqliteDatabase {
    async fn find_all_watchers_by_job_name_and_status(
        &self,
        job_name: &str, // TODO: newtype pattern
        status: JobDoneWatcherStatus
    ) -> anyhow::Result<Vec<JobDoneWatcher>> {
        todo!()
    }

    async fn find_all_watchers(&self) -> anyhow::Result<Vec<JobDoneWatcher>> {
        let mut conn = self.acquire()
            .await
            .with_context(|| "Unable to acquire a database connection".to_string())?;

        let job_done_watcher_entities: Vec<JobDoneWatcherEntity> = sqlx::query_as!(JobDoneWatcherEntity, r#"
            SELECT
                job_done_watchers.id,
                job_done_watchers.job_name,
                job_done_watchers.timeout_seconds,
                job_done_watchers.status,
                job_done_watchers.created_at,
                coalesce(json_group_array(json_object(
                    'id', job_done_trigger_webhooks.id,
                    'webhook_id', job_done_trigger_webhooks.webhook_id,
                    'timeout_seconds', job_done_trigger_webhooks.timeout_seconds,
                    'status', job_done_trigger_webhooks.status,
                    'called_at', job_done_trigger_webhooks.called_at)), json_object()) AS "job_done_trigger_webhooks!: String"
            FROM
                job_done_watchers
            LEFT JOIN
                job_done_trigger_webhooks ON job_done_watchers.id = job_done_trigger_webhooks.job_done_watcher_id
            GROUP BY
                job_done_watchers.id
        "#).fetch_all(&mut *conn).await?;

        Ok(job_done_watcher_entities.into_iter().map(JobDoneWatcher::from).collect())
    }

    async fn find_watcher_by_id(&self, id: &str) -> anyhow::Result<Option<JobDoneWatcher>> {
        todo!()
    async fn find_watcher_by_id(&self, id: &Uuid) -> anyhow::Result<Option<JobDoneWatcher>> {
    }

    async fn create_watcher(&self, job_done_watcher: &JobDoneWatcher) -> anyhow::Result<()> {
        todo!()
    }
}