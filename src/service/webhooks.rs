use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use chrono::Utc;
use uuid::Uuid;

use crate::models::{CreateWebhookRequest, Webhook};

pub static WEBHOOKS:
    LazyLock<Mutex<HashMap<String, Webhook>>> =
        LazyLock::new(|| {
            Mutex::new(HashMap::with_capacity(10))
        });

pub async fn create_webhook(webhook: CreateWebhookRequest) -> Webhook {
    let webhook = Webhook {
        id: Uuid::new_v4().to_string(),
        url: webhook.url,
        request_body: webhook.request_body,
        description: webhook.description,
        created_at: Utc::now(),
    };

    match WEBHOOKS.lock() {
        Ok(mut webhooks) => webhooks.insert(webhook.id.clone(), webhook.clone()),
        Err(_) => panic!(), // TODO:
    };

    webhook
}

pub async fn get_webhooks() -> Vec<Webhook> {
    match WEBHOOKS.lock() {
        Ok(webhooks) => webhooks.values().cloned().collect(),
        Err(_) => panic!(),
    }
}

pub async fn get_webhooks_by_id(webhook_id: &str) -> Option<Webhook> {
    match WEBHOOKS.lock() {
        Ok(webhooks) => webhooks.get(webhook_id).cloned(),
        Err(_) => panic!(), // TODO:
    }
}