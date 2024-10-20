use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use moka::sync::Cache;

use crate::models::Webhook;
use crate::repository::{SqliteDatabase, SqlxAcquire};

#[async_trait]
pub trait WebhookRepository: Send + Sync {
    async fn find_all(&self) -> anyhow::Result<Vec<Webhook>>;
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Webhook>>;
    async fn save(&self, entity: Webhook) -> anyhow::Result<()>;
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
    async fn find_all(&self) -> anyhow::Result<Vec<Webhook>> {
        Ok(self.webhook_by_id.iter()
            .map(|(_, webhook)| webhook)
            .collect())
    }

    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Webhook>> {
        Ok(self.webhook_by_id.get(id))
    }

    async fn save(&self, webhook: Webhook) -> anyhow::Result<()> {
        Ok(self.webhook_by_id.insert(webhook.id.clone(), webhook))
    }
}

#[async_trait]
impl WebhookRepository for SqliteDatabase {
    async fn find_all(&self) -> anyhow::Result<Vec<Webhook>> {
        let mut conn = self.acquire().await?;
        sqlx::query_as!(Webhook, "SELECT * FROM webhooks")
    }

    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Webhook>> {
        todo!()
    }

    async fn save(&self, webhook: Webhook) -> anyhow::Result<()> {
        todo!()
    }
}