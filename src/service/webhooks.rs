use chrono::Utc;
use uuid::Uuid;

use crate::models::service::{CreateWebhookRequest, Webhook};
use crate::repository;

pub async fn create_webhook(create_webhook_request: CreateWebhookRequest) -> anyhow::Result<Webhook> {
    log::info!("Creating a new webhook with URL: {}", create_webhook_request.url());

    let webhook = Webhook::new(
        Uuid::new_v4(),
        create_webhook_request.url().clone(),
        create_webhook_request.request_body(),
        create_webhook_request.description(),
        Utc::now(),
    );

    let webhook_repository = repository::get_webhook_repository();
    match webhook_repository.create_webhook(&webhook).await {
        Ok(()) => {
            log::info!("Successfully created webhook with ID: {}", webhook.id());
            Ok(webhook)
        }
        Err(error) => {
            log::error!("Failed to create webhook: {:?}", error);
            Err(error)
        }
    }
}

pub async fn get_webhooks() -> anyhow::Result<Vec<Webhook>> {
    log::info!("Fetching all webhooks");

    let webhook_repository = repository::get_webhook_repository();

    match webhook_repository.find_all_webhooks().await {
        Ok(webhooks) => {
            log::info!("Successfully retrieved {} webhooks", webhooks.len());
            Ok(webhooks)
        }
        Err(error) => {
            log::error!("Failed to fetch webhooks: {:?}", error);
            Err(error)
        }
    }
}

pub async fn get_webhook_by_id(webhook_id: &Uuid) -> anyhow::Result<Option<Webhook>> {
    log::info!("Fetching webhook with ID: {}", webhook_id);

    let webhook_repository = repository::get_webhook_repository();

    match webhook_repository.find_webhook_by_id(webhook_id).await {
        Ok(Some(webhook)) => {
            log::info!("Successfully retrieved webhook with ID: {}", webhook.id());
            Ok(Some(webhook))
        }
        Ok(None) => {
            log::warn!("Webhook with ID {} not found", webhook_id);
            Ok(None)
        }
        Err(error) => {
            log::error!("Failed to fetch webhook by ID {}: {:?}", webhook_id, error);
            Err(error)
        }
    }
}