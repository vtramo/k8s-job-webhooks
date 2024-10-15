use chrono::Utc;
use uuid::Uuid;

use crate::models::{CreateWebhookRequest, Webhook};
use crate::repository;

pub async fn create_webhook(webhook: CreateWebhookRequest) -> Webhook {
    let webhook = Webhook {
        id: Uuid::new_v4().to_string(),
        url: webhook.url,
        request_body: webhook.request_body,
        description: webhook.description,
        created_at: Utc::now(),
    };

    let webhook_repository = repository::get_webhook_repository();
    webhook_repository.save(webhook.clone());

    webhook
}

pub async fn get_webhooks() -> Vec<Webhook> {
    let webhook_repository = repository::get_webhook_repository();
    webhook_repository.find_all()
}

pub async fn get_webhooks_by_id(webhook_id: &str) -> Option<Webhook> {
    let webhook_repository = repository::get_webhook_repository();
    webhook_repository.find_by_id(webhook_id)
}