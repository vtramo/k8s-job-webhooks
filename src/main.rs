use actix_web::{App, HttpServer};

use k8s_job_webhooks::controller;
use k8s_job_webhooks::service::k8s_job_watcher;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    actix_web::rt::spawn(k8s_job_watcher::watch_jobs());

    HttpServer::new(|| {
        App::new()
            .service(controller::webhooks::post_webhooks)
            .service(controller::webhooks::get_webhooks)
            .service(controller::job_done_watchers::post_job_done_watchers)
            .service(controller::job_done_watchers::get_job_done_watchers)
            .service(controller::job_done_watchers::get_job_done_watcher)
    }).bind(("0.0.0.0", 8080))?
        .run()
        .await
}