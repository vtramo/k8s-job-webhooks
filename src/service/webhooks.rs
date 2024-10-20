use chrono::Utc;
use uuid::Uuid;

use crate::models::{CreateWebhookRequest, Webhook};
use crate::repository;

pub async fn create_webhook(webhook: CreateWebhookRequest) -> anyhow::Result<Webhook> {
    let webhook = Webhook {
        id: Uuid::new_v4(),
        url: webhook.url,
        request_body: webhook.request_body,
        description: webhook.description,
        created_at: Utc::now(),
    };

    let webhook_repository = repository::get_webhook_repository();
    match webhook_repository.create_webhook(&webhook).await {
        Ok(()) => Ok(webhook),
        Err(error) => Err(error)
    }
}

pub async fn get_webhooks() -> anyhow::Result<Vec<Webhook>> {
    let webhook_repository = repository::get_webhook_repository();
    webhook_repository.find_all_webhooks().await
}

pub async fn get_webhook_by_id(webhook_id: &Uuid) -> anyhow::Result<Option<Webhook>> {
    let webhook_repository = repository::get_webhook_repository();
    webhook_repository.find_webhook_by_id(webhook_id).await
}