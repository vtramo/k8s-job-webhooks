use actix_web::{get, HttpResponse, post, Responder, web};
use crate::models::CreateWebhookRequest;
use crate::service::webhooks;

#[post("/webhooks")]
pub async fn post_webhooks(webhook: web::Json<CreateWebhookRequest>) -> impl Responder {
    let webhook = webhook.0;
    match webhooks::create_webhook(webhook).await {
        Ok(created_webhook) => HttpResponse::Created().json(created_webhook),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/webhooks")]
pub async fn get_webhooks() -> impl Responder {
    match webhooks::get_webhooks().await {
        Ok(webhooks) => HttpResponse::Ok().json(webhooks),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}