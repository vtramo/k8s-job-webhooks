use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use moka::sync::Cache;

use crate::models::Webhook;

#[async_trait]
pub trait WebhookRepository: Send + Sync {
    async fn find_all(&self) -> Vec<Webhook>;
    async fn find_by_id(&self, id: &str) -> Option<Webhook>;
    async fn save(&self, entity: Webhook);
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
    async fn find_all(&self) -> Vec<Webhook> {
        self.webhook_by_id.iter()
            .map(|(_, webhook)| webhook)
            .collect()
    }

    async fn find_by_id(&self, id: &str) -> Option<Webhook> {
        self.webhook_by_id.get(id)
    }

    async fn save(&self, webhook: Webhook) {
        self.webhook_by_id.insert(webhook.id.clone(), webhook);
    }
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