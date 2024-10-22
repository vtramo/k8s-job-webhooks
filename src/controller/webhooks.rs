use actix_web::{get, HttpResponse, post, Responder, web};
use uuid::Uuid;

use crate::models::CreateWebhookRequest;
use crate::service;

#[post("/webhooks")]
pub async fn post_webhooks(webhook: web::Json<CreateWebhookRequest>) -> impl Responder {
    let webhook = webhook.0;
    match service::webhooks::create_webhook(webhook).await {
        Ok(created_webhook) => HttpResponse::Created().json(created_webhook),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/webhooks")]
pub async fn get_webhooks() -> impl Responder {
    match service::webhooks::get_webhooks().await {
        Ok(webhooks) => HttpResponse::Ok().json(webhooks),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/webhooks/{id}")]
pub async fn get_webhook_by_id(id: web::Path<String>) -> impl Responder {
    let webhook_id = match Uuid::parse_str(id.as_str()) {
        Ok(webhook_id) => webhook_id,
        Err(_) => {
            log::warn!("Invalid UUID format: {}", id);
            return HttpResponse::BadRequest().finish();
        },
    };

    match service::webhooks::get_webhook_by_id(&webhook_id).await {
        Ok(option_webhook) => match option_webhook {
            None => HttpResponse::NotFound().finish(),
            Some(job_done_watcher) => HttpResponse::Ok().json(job_done_watcher),
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}