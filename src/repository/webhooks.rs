use std::sync::{Arc, OnceLock};

use anyhow::Context;
use async_trait::async_trait;
use moka::sync::Cache;
use uuid::Uuid;

use crate::models::entity::WebhookEntity;
use crate::models::service::Webhook;
use crate::repository::{SqliteDatabase, SqlxAcquire};

#[async_trait]
pub trait WebhookRepository: Send + Sync {
    async fn find_all_webhooks(&self) -> anyhow::Result<Vec<Webhook>>;
    async fn find_webhook_by_id(&self, uuid: &Uuid) -> anyhow::Result<Option<Webhook>>;
    async fn create_webhook(&self, webhook: &Webhook) -> anyhow::Result<()>;
}

pub static WEBHOOK_REPOSITORY: OnceLock<Arc<dyn WebhookRepository>> = OnceLock::new();

pub fn set_webhook_repository(webhook_repository: impl WebhookRepository + 'static) {
    if let Err(_) = WEBHOOK_REPOSITORY.set(Arc::new(webhook_repository)) {
        panic!("You can't set Webhook Repository twice!");
    }
}

pub fn get_webhook_repository() -> Arc<dyn WebhookRepository> {
    WEBHOOK_REPOSITORY.get().expect("Should be set!").clone()
}

pub struct InMemoryWebhookRepository {
    webhook_by_id: Cache<String, Webhook>,
}

impl InMemoryWebhookRepository {
    pub fn new() -> Self {
        Self {
            webhook_by_id: Cache::new(15),
        }
    }
}

#[async_trait]
impl WebhookRepository for InMemoryWebhookRepository {
    async fn find_all_webhooks(&self) -> anyhow::Result<Vec<Webhook>> {
        Ok(self.webhook_by_id.iter()
            .map(|(_, webhook)| webhook)
            .collect())
    }

    async fn find_webhook_by_id(&self, uuid: &Uuid) -> anyhow::Result<Option<Webhook>> {
        Ok(self.webhook_by_id.get(&uuid.to_string()))
    }

    async fn create_webhook(&self, webhook: &Webhook) -> anyhow::Result<()> {
        Ok(self.webhook_by_id.insert(webhook.id().to_string(), webhook.clone()))
    }
}

#[async_trait]
impl WebhookRepository for SqliteDatabase {
    async fn find_all_webhooks(&self) -> anyhow::Result<Vec<Webhook>> {
        let mut conn = self.acquire()
            .await
            .with_context(|| "Unable to acquire a database connection".to_string())?;

        let webhook_entities: Vec<WebhookEntity> = sqlx::query_file_as!(WebhookEntity, "queries/sqlite/find_all_webhooks.sql")
            .fetch_all(&mut *conn)
            .await?;

        Ok(webhook_entities.iter().map(Webhook::from).collect())
    }

    async fn find_webhook_by_id(&self, uuid: &Uuid) -> anyhow::Result<Option<Webhook>> {
        let mut conn = self.acquire()
            .await
            .with_context(|| "Unable to acquire a database connection".to_string())?;

        let uuid = uuid.to_string();
        Ok(sqlx::query_file_as!(WebhookEntity, "queries/sqlite/find_webhook_by_id.sql", uuid)
            .fetch_optional(&mut *conn)
            .await?
            .map(Webhook::from))
    }

    async fn create_webhook(&self, webhook: &Webhook) -> anyhow::Result<()> {
        let mut conn = self.acquire().await?;

        let now = chrono::Utc::now();
        let webhook_id = webhook.id().to_string();
        let webhook_url = webhook.url().to_string();
        let webhook_request_body = webhook.request_body();
        let webhook_description = webhook.description();
        sqlx::query!(
            r#"
                INSERT INTO webhooks ( id, url, request_body, description, created_at )
                VALUES ( ?1, ?2, ?3, ?4, ?5 )
            "#,
            webhook_id,
            webhook_url,
            webhook_request_body,
            webhook_description,
            now
        ).execute(&mut *conn)
         .await?;

        Ok(())
    }
}