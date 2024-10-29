use std::sync::{Arc, OnceLock};

use anyhow::Context;
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::models::entity::JobFamilyWatcherEntity;
use crate::models::service::JobFamilyWatcher;
use crate::repository::{SqliteDatabase, SqlxAcquire};

static JOB_FAMILY_WATCHER_REPOSITORY: OnceLock<Arc<dyn JobFamilyWatcherRepository>> = OnceLock::new();

pub fn set_job_family_watcher_repository(job_family_watcher_repository: impl JobFamilyWatcherRepository + 'static) {
    if let Err(_) = JOB_FAMILY_WATCHER_REPOSITORY.set(Arc::new(job_family_watcher_repository)) {
        panic!("You can't set Webhook Repository twice!");
    }
}

pub fn get_job_family_watcher_repository() -> Arc<dyn JobFamilyWatcherRepository> {
    Arc::clone(JOB_FAMILY_WATCHER_REPOSITORY.get().expect("Should be set!"))
}

#[async_trait]
pub trait JobFamilyWatcherRepository: Send + Sync {
    async fn create_job_family_watcher(&self, job_family_watcher: JobFamilyWatcher) -> anyhow::Result<()>;
    async fn find_all_job_family_watchers_by_job_family(&self, job_family: &str) -> anyhow::Result<Vec<JobFamilyWatcher>>;
}

#[async_trait]
impl JobFamilyWatcherRepository for SqliteDatabase {
    async fn create_job_family_watcher(&self, job_family_watcher: JobFamilyWatcher) -> anyhow::Result<()> {
        let mut conn = self.acquire()
            .await
            .with_context(|| "Unable to acquire a database connection".to_string())?;

        let id = Uuid::new_v4().to_string();
        let job_family = job_family_watcher.job_family();
        let url = job_family_watcher.url().to_string();
        let request_body = job_family_watcher.request_body();
        let description = job_family_watcher.description();
        let created_at = Utc::now();
        sqlx::query_file!("queries/sqlite/insert_job_family_watcher.sql",
            id,
            job_family,
            url,
            request_body,
            description,
            created_at
        ).execute(&mut *conn).await?;

        Ok(())
    }

    async fn find_all_job_family_watchers_by_job_family(&self, job_family: &str) -> anyhow::Result<Vec<JobFamilyWatcher>> {
        let mut conn = self.acquire()
            .await
            .with_context(|| "Unable to acquire a database connection".to_string())?;

        let job_family_watcher_entities: Vec<_> =
            sqlx::query_file_as!(JobFamilyWatcherEntity,
                "queries/sqlite/find_all_job_family_watchers_by_job_family.sql",
                job_family
            ).fetch_all(&mut *conn)
            .await?;

        Ok(job_family_watcher_entities.into_iter().map(JobFamilyWatcher::from).collect())
    }
}
