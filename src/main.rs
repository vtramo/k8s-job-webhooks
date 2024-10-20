use actix_web::{App, HttpServer, web};

use k8s_job_webhooks::controller;
use k8s_job_webhooks::controller::IdempotencyMap;
use k8s_job_webhooks::repository;
use k8s_job_webhooks::service::k8s_job_watcher;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    repository::set_webhook_repository(repository::InMemoryWebhookRepository::new());
    repository::set_job_done_watcher_repository(repository::InMemoryJobDoneWatcherRepository::new());

    actix_web::rt::spawn(k8s_job_watcher::watch_jobs());

    let app_state_idempotency_map = web::Data::new(IdempotencyMap::new());
    HttpServer::new(move || {
        App::new()
            .app_data(app_state_idempotency_map.clone())
            .service(controller::webhooks::post_webhooks)
            .service(controller::webhooks::get_webhooks)
            .service(controller::webhooks::get_webhook_by_id)
            .service(controller::job_done_watchers::post_job_done_watchers)
            .service(controller::job_done_watchers::get_job_done_watchers)
            .service(controller::job_done_watchers::get_job_done_watcher)
    }).bind(("0.0.0.0", 8080))?
        .run()
        .await
}