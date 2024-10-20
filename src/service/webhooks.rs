use chrono::Utc;
use uuid::Uuid;

use crate::models::{CreateWebhookRequest, Webhook};
use crate::repository;

pub async fn create_webhook(webhook: CreateWebhookRequest) -> anyhow::Result<Webhook> {
    let webhook = Webhook {
        id: Uuid::new_v4().to_string(),
        url: webhook.url,
        request_body: webhook.request_body,
        description: webhook.description,
        created_at: Utc::now(),
    };

    let webhook_repository = repository::get_webhook_repository();
    match webhook_repository.save(webhook.clone()).await {
        Ok(()) => Ok(webhook),
        Err(error) => Err(error)
    }
}

pub async fn get_webhooks() -> anyhow::Result<Vec<Webhook>> {
    let webhook_repository = repository::get_webhook_repository();
    webhook_repository.find_all().await
}

pub async fn get_webhooks_by_id(webhook_id: &str) -> anyhow::Result<Option<Webhook>> {
    let webhook_repository = repository::get_webhook_repository();
    webhook_repository.find_by_id(webhook_id).await
}