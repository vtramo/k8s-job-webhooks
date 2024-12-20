use std::ops::Deref;
use std::sync::{Arc, OnceLock};

use anyhow::{anyhow, Context};
use async_rwlock::RwLock;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures_util::stream;
use futures_util::StreamExt;
use moka::sync::Cache;
use sqlx::Acquire;
use uuid::Uuid;

use crate::models::entity::JobDoneWatcherEntity;
use crate::models::service::{JobDoneTriggerWebhookStatus, JobDoneWatcher, JobDoneWatcherStatus, JobName};
use crate::repository::{SqliteDatabase, SqlxAcquire};

#[async_trait]
pub trait JobDoneWatcherRepository: Send + Sync {
    async fn find_all_watchers_by_job_name_and_status(
        &self,
        job_name: &JobName,
        status: JobDoneWatcherStatus
    ) -> anyhow::Result<Vec<JobDoneWatcher>>;
    async fn find_all_watchers(&self) -> anyhow::Result<Vec<JobDoneWatcher>>;
    async fn find_watcher_by_id(&self, id: &Uuid) -> anyhow::Result<Option<JobDoneWatcher>>;
    async fn create_watcher(&self, job_done_watcher: &JobDoneWatcher) -> anyhow::Result<()>;
    async fn update_watcher_status(&self, id: &Uuid, job_done_watcher_status: JobDoneWatcherStatus) -> anyhow::Result<()>;
    async fn update_watcher_status_by_status(
        &self,
        id: &Uuid,
        status: JobDoneWatcherStatus,
        new_status: JobDoneWatcherStatus
    ) -> anyhow::Result<()>;
    async fn update_watchers_status_by_job_name_and_status(
        &self,
        job_name: &JobName,
        status: JobDoneWatcherStatus,
        new_status: JobDoneWatcherStatus
    ) -> anyhow::Result<Vec<JobDoneWatcher>>;
    async fn update_job_done_trigger_webhook_status_and_called_at(
        &self,
        id: &Uuid,
        job_done_trigger_webhook_id: &Uuid,
        job_done_trigger_webhook_status: JobDoneTriggerWebhookStatus,
        job_done_trigger_webhook_called_at: DateTime<Utc>,
    ) -> anyhow::Result<()>;
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

#[async_trait]
impl JobDoneWatcherRepository for InMemoryJobDoneWatcherRepository {
    async fn find_all_watchers_by_job_name_and_status(
        &self,
        job_name: &JobName,
        status: JobDoneWatcherStatus
    ) -> anyhow::Result<Vec<JobDoneWatcher>> {
        Ok(stream::iter(self.job_done_watcher_by_id.iter())
            .filter_map(|(_, job_done_watcher): (_, Arc<RwLock<JobDoneWatcher>>)| {
                let job_done_watcher = Arc::clone(&job_done_watcher);
                async move {
                    let job_done_watcher = job_done_watcher.read().await;
                    if job_done_watcher.job_name() == job_name.as_str() && job_done_watcher.status() == status {
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
        self.job_done_watcher_by_id.insert(job_done_watcher.id().to_string(), Arc::new(RwLock::new(job_done_watcher.clone())));
        Ok(())
    }

    async fn update_watcher_status(&self, id: &Uuid, job_done_watcher_status: JobDoneWatcherStatus) -> anyhow::Result<()> {
        let id = id.to_string();
        if let Some(job_done_watcher) = self.job_done_watcher_by_id.get(&id) {
            let mut job_done_watcher = job_done_watcher.write().await;
            job_done_watcher.set_status(job_done_watcher_status);
            Ok(())
        } else {
            return Err(anyhow!("Job Done Watcher with id {} not found!", id));
        }
    }

    async fn update_watcher_status_by_status(
        &self,
        id: &Uuid,
        status: JobDoneWatcherStatus,
        new_status: JobDoneWatcherStatus
    ) -> anyhow::Result<()> {
        let id_str = id.to_string();
        if let Some(job_done_watcher) = self.job_done_watcher_by_id.get(&id_str) {
            let mut watcher = job_done_watcher.write().await;

            if watcher.status() == status {
                watcher.set_status(new_status);
            }

            Ok(())
        } else {
            Err(anyhow!("Job Done Watcher with id {} not found!", id_str))
        }
    }

    async fn update_watchers_status_by_job_name_and_status(
        &self,
        job_name: &JobName,
        status: JobDoneWatcherStatus,
        new_status: JobDoneWatcherStatus
    ) -> anyhow::Result<Vec<JobDoneWatcher>> {
        let mut updated_watchers = Vec::new();

        for (_, job_done_watcher) in &self.job_done_watcher_by_id {
            let job_done_watcher = Arc::clone(&job_done_watcher);
            let mut watcher = job_done_watcher.write().await;

            if watcher.job_name() == job_name.as_str() && watcher.status() == status {
                watcher.set_status(new_status);
                updated_watchers.push(watcher.clone());
            }
        }

        Ok(updated_watchers)
    }

    async fn update_job_done_trigger_webhook_status_and_called_at(
        &self,
        id: &Uuid,
        job_done_trigger_webhook_id: &Uuid,
        job_done_trigger_webhook_status: JobDoneTriggerWebhookStatus,
        job_done_trigger_webhook_called_at: DateTime<Utc>
    ) -> anyhow::Result<()> {
        let id_str = id.to_string();

        if let Some(job_done_watcher) = self.job_done_watcher_by_id.get(&id_str) {
            let mut watcher = job_done_watcher.write().await;

            if let Some(trigger_webhook) = watcher
                .job_done_trigger_webhooks_mut()
                .into_iter()
                .find(|wh| wh.id() == *job_done_trigger_webhook_id)
            {
                trigger_webhook.set_status(job_done_trigger_webhook_status);
                trigger_webhook.set_called_at(job_done_trigger_webhook_called_at);
                Ok(())
            } else {
                Err(anyhow!(
                    "Job Done Trigger Webhook with id {} not found for Job Done Watcher {}",
                    job_done_trigger_webhook_id,
                    id
                ))
            }
        } else {
            Err(anyhow!("Job Done Watcher with id {} not found!", id))
        }
    }
}

#[async_trait::async_trait]
impl JobDoneWatcherRepository for SqliteDatabase {
    async fn find_all_watchers_by_job_name_and_status(
        &self,
        job_name: &JobName,
        status: JobDoneWatcherStatus
    ) -> anyhow::Result<Vec<JobDoneWatcher>> {
        let mut conn = self.acquire()
            .await
            .with_context(|| "Unable to acquire a database connection".to_string())?;

        let job_name = job_name.to_string();
        let status = status.to_string();
        let job_done_watcher_entities: Vec<JobDoneWatcherEntity> =
            sqlx::query_file_as!(JobDoneWatcherEntity,
                "queries/sqlite/find_all_watchers_by_job_name_and_status.sql",
                job_name,
                status
            ).fetch_all(&mut *conn)
             .await?;

        Ok(job_done_watcher_entities.into_iter().map(JobDoneWatcher::from).collect())
    }

    async fn find_all_watchers(&self) -> anyhow::Result<Vec<JobDoneWatcher>> {
        let mut conn = self.acquire()
            .await
            .with_context(|| "Unable to acquire a database connection".to_string())?;


        let job_done_watcher_entities: Vec<JobDoneWatcherEntity> =
            sqlx::query_file_as!(JobDoneWatcherEntity, "queries/sqlite/find_all_watchers.sql")
                .fetch_all(&mut *conn)
                .await?;

        Ok(job_done_watcher_entities.into_iter().map(JobDoneWatcher::from).collect())
    }

    async fn find_watcher_by_id(&self, id: &Uuid) -> anyhow::Result<Option<JobDoneWatcher>> {
        let mut conn = self.acquire()
            .await
            .with_context(|| "Unable to acquire a database connection".to_string())?;

        let id = id.to_string();
        let job_done_watcher_entity: Option<JobDoneWatcherEntity> =
            sqlx::query_file_as!(JobDoneWatcherEntity, "queries/sqlite/find_watcher_by_id.sql", id)
                .fetch_optional(&mut *conn).await?;

        Ok(job_done_watcher_entity.map(JobDoneWatcher::from))
    }

    async fn create_watcher(&self, job_done_watcher: &JobDoneWatcher) -> anyhow::Result<()> {
        let mut conn = self.acquire()
            .await
            .with_context(|| "Unable to acquire a database connection".to_string())?;

        let mut tx = conn.begin().await?;

        let job_done_watcher_id = job_done_watcher.id().to_string();
        let job_done_watcher_job_name = job_done_watcher.job_name();
        let job_done_watcher_timeout_seconds = job_done_watcher.timeout_seconds();
        let job_done_watcher_status = job_done_watcher.status().to_string();
        let job_done_watcher_created_at = job_done_watcher.created_at();

        sqlx::query_file!("queries/sqlite/insert_job_done_watcher.sql",
            job_done_watcher_id,
            job_done_watcher_job_name,
            job_done_watcher_timeout_seconds,
            job_done_watcher_status,
            job_done_watcher_created_at
        ).execute(&mut *tx)
         .await?;


        let trigger_webhook_values: Vec<_> = job_done_watcher
            .job_done_trigger_webhooks()
            .iter()
            .map(|job_done_trigger_webhook| {
                (
                    job_done_trigger_webhook.id().to_string(),
                    job_done_trigger_webhook.webhook_id().to_string(),
                    job_done_watcher_id.clone(),
                    job_done_trigger_webhook.timeout_seconds(),
                    job_done_trigger_webhook.status().to_string(),
                )
            })
            .collect();

        if !trigger_webhook_values.is_empty() {
            let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO job_done_trigger_webhooks (id, webhook_id, job_done_watcher_id, timeout_seconds, status)"
            );

            query_builder.push_values(
                trigger_webhook_values,
                |mut builder, (id, webhook_id, job_done_watcher_id, timeout_seconds, status)| {
                    builder.push_bind(id)
                        .push_bind(webhook_id)
                        .push_bind(job_done_watcher_id)
                        .push_bind(timeout_seconds)
                        .push_bind(status);
                }
            );

            query_builder.build().execute(&mut *tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn update_watcher_status(&self, id: &Uuid, new_status: JobDoneWatcherStatus) -> anyhow::Result<()> {
        let mut conn = self.acquire()
            .await
            .with_context(|| "Unable to acquire a database connection".to_string())?;

        let mut tx = conn.begin().await?;

        let id = id.to_string();
        let new_status = new_status.to_string();
        let _ = sqlx::query_file!("queries/sqlite/update_watcher_status.sql", id, new_status).execute(&mut *tx).await?;

        tx.commit().await?;

        Ok(())
    }

    async fn update_watcher_status_by_status(
        &self,
        id: &Uuid,
        status: JobDoneWatcherStatus,
        new_status: JobDoneWatcherStatus
    ) -> anyhow::Result<()> {
        let mut conn = self.acquire()
            .await
            .with_context(|| "Unable to acquire a database connection".to_string())?;

        let mut tx = conn.begin().await?;

        let id = id.to_string();
        let status = status.to_string();
        let new_status = new_status.to_string();
        let _ = sqlx::query_file!("queries/sqlite/update_watcher_status_by_status.sql", id, status, new_status).execute(&mut *tx).await?;

        tx.commit().await?;

        Ok(())
    }

    async fn update_watchers_status_by_job_name_and_status(
        &self,
        job_name: &JobName,
        status: JobDoneWatcherStatus,
        new_status: JobDoneWatcherStatus
    ) -> anyhow::Result<Vec<JobDoneWatcher>> {
        let mut conn = self.acquire()
            .await
            .with_context(|| "Unable to acquire a database connection".to_string())?;

        let mut tx = conn.begin().await?;

        struct Id { id: String }
        impl Deref for Id {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                &self.id
            }
        }
        let job_name = job_name.to_string();
        let status = status.to_string();
        let new_status = new_status.to_string();
        let ids: Vec<String> = sqlx::query_file_as!(
            Id,
            "queries/sqlite/update_watchers_status_by_job_name_and_status.sql",
            job_name, status, new_status
        ).fetch_all(&mut *tx).await?
            .iter()
            .map(|id| id.to_string())
            .collect();

        let updated_job_done_watchers: Vec<JobDoneWatcher> = sqlx::query_file_as!(
            JobDoneWatcherEntity,
            "queries/sqlite/find_all_watchers_by_job_name_and_status.sql",
            job_name, new_status
        ).fetch_all(&mut *tx).await?
            .into_iter()
            .filter(|job_done_watcher| ids.contains(&job_done_watcher.id))
            .map(JobDoneWatcher::from)
            .collect(); // I'm currently unable to construct a SQL query using the IN clause with sqlx and SQLite. Any suggestions are welcome!

        tx.commit().await?;

        Ok(updated_job_done_watchers)
    }

    async fn update_job_done_trigger_webhook_status_and_called_at(
        &self,
        id: &Uuid,
        job_done_trigger_webhook_id: &Uuid,
        job_done_trigger_webhook_status: JobDoneTriggerWebhookStatus,
        job_done_trigger_webhook_called_at: DateTime<Utc>
    ) -> anyhow::Result<()> {
        let mut conn = self.acquire()
            .await
            .with_context(|| "Unable to acquire a database connection".to_string())?;

        let id = id.to_string();
        let job_done_trigger_webhook_id = job_done_trigger_webhook_id.to_string();
        let job_done_trigger_webhook_status = job_done_trigger_webhook_status.to_string();
        let called_at = job_done_trigger_webhook_called_at;
        sqlx::query!(r#"
            UPDATE job_done_trigger_webhooks
            SET (status, called_at) = (?3, ?4)
            WHERE
                job_done_trigger_webhooks.id = ?2
            AND
                job_done_trigger_webhooks.job_done_watcher_id = ?1
        "#, id, job_done_trigger_webhook_id, job_done_trigger_webhook_status, called_at)
            .execute(&mut *conn).await?;

        Ok(())
    }
}