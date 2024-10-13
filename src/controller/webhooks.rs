use actix_web::{get, HttpResponse, post, Responder, web};
use crate::models::CreateWebhookRequest;
use crate::service::webhooks;

#[post("/webhooks")]
pub async fn post_webhooks(webhook: web::Json<CreateWebhookRequest>) -> impl Responder {
    let webhook = webhook.0;
    let created_webhook = webhooks::create_webhook(webhook).await;
    HttpResponse::Created().json(created_webhook)
}

#[get("/webhooks")]
pub async fn get_webhooks() -> impl Responder {
    let webhooks = webhooks::get_webhooks().await;
    HttpResponse::Ok().json(webhooks)
}