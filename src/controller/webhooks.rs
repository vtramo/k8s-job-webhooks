use actix_web::{get, HttpResponse, post, Responder, web};
use uuid::Uuid;

use crate::models::api::{CreateWebhookRequestApi, WebhookApi};
use crate::service;

#[post("/webhooks")]
pub async fn post_webhooks(webhook: web::Json<CreateWebhookRequestApi>) -> impl Responder {
    let create_webhook_request = webhook.0.try_into().unwrap(); // TODO:

    match service::webhooks::create_webhook(create_webhook_request).await {
        Ok(created_webhook) => HttpResponse::Created()
            .json(WebhookApi::from(&created_webhook)),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/webhooks")]
pub async fn get_webhooks() -> impl Responder {
    match service::webhooks::get_webhooks().await {
        Ok(webhooks) => HttpResponse::Ok()
            .json(webhooks.iter()
                .map(WebhookApi::from)
                .collect::<Vec<WebhookApi>>()),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/webhooks/{id}")]
pub async fn get_webhook_by_id(id: web::Path<String>) -> impl Responder {
    let webhook_id = match Uuid::parse_str(id.as_str()) {
        Ok(webhook_id) => webhook_id,
        Err(_) => {
            log::warn!("Invalid UUID format: {}", id); // TODO:
            return HttpResponse::BadRequest().finish();
        },
    };

    match service::webhooks::get_webhook_by_id(&webhook_id).await {
        Ok(option_webhook) => match option_webhook {
            None => HttpResponse::NotFound().finish(),
            Some(webhook) => HttpResponse::Ok()
                .json(WebhookApi::from(&webhook)),
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}