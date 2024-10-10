use actix_web::{App, HttpServer};

use k8s_cronjob_monitor::controller;
use k8s_cronjob_monitor::k8s_job_watcher;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    actix_web::rt::spawn(k8s_job_watcher::watch_jobs());

    HttpServer::new(|| {
        App::new()
            .service(controller::post_cronjobs_monitors)
            .service(controller::post_jobs_monitors)
    }).bind(("0.0.0.0", 8080))?
        .run()
        .await
}