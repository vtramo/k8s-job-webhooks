use std::env;
use actix_web::{App, HttpServer, web};
use actix_web::middleware::Logger;
use crate::controller::IdempotencyMap;

use crate::{controller, repository};

pub fn init_logging() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    log::info!("Logging initialized.");
    Ok(())
}

pub async fn init_database() -> anyhow::Result<()> {
    log::info!("Init database...");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let database = database_url.split(':').next().unwrap_or("");

    match database {
        "sqlite" => {
            let repository = repository::SqliteDatabase::connect(&database_url).await?;
            repository::set_webhook_repository(repository.clone());
            repository::set_job_done_watcher_repository(repository);
            Ok(())
        },
        _ => Err(anyhow::anyhow!("Unsupported database: {}", database))
    }
}

pub async fn init_http_server() -> anyhow::Result<()> {
    log::info!("Init http server...");

    let app_state_idempotency_map = web::Data::new(IdempotencyMap::new());
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%r - %a - %{User-Agent}i - Response Status Code: %s"))
            .app_data(app_state_idempotency_map.clone())
            .service(controller::webhooks::post_webhooks)
            .service(controller::webhooks::get_webhooks)
            .service(controller::webhooks::get_webhook_by_id)
            .service(controller::job_done_watchers::post_job_done_watchers)
            .service(controller::job_done_watchers::get_job_done_watchers)
            .service(controller::job_done_watchers::get_job_done_watcher)
    }).bind(("0.0.0.0", 8080))?
        .run()
        .await?;

    Ok(())
}