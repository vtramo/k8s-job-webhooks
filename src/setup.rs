use std::env;
use std::fs::read_to_string;

use actix_web::{App, HttpServer, web};
use actix_web::middleware::Logger;
use futures_util::stream;
use futures_util::StreamExt;
use sqlx::sqlite::SqlitePoolOptions;
use yaml_rust2::YamlLoader;

use crate::{controller, repository, service};
use crate::controller::IdempotencyMap;
use crate::models::JobFamilyWatcher;
use crate::repository::SqlxAcquire;

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

            let repository = if is_in_memory_sqlite(&database_url) {
                let repository = repository::SqliteDatabase::connect_in_memory(&database_url).await?;
                log::info!("Running migrations for in-memory SQLite database...");

                let mut conn = repository.acquire().await?;
                sqlx::migrate!("migrations/sqlite")
                    .run(&mut *conn)
                    .await?;

                log::info!("Migrations completed successfully.");
                repository
            } else {
                repository::SqliteDatabase::connect(&database_url).await?
            };

            repository::set_webhook_repository(repository.clone());
            repository::set_job_done_watcher_repository(repository.clone());
            repository::set_job_family_watcher_repository(repository);

            Ok(())
        },
        _ => Err(anyhow::anyhow!("Unsupported database: {}", database))
    }
}

fn is_in_memory_sqlite(url: &str) -> bool {
    const URL_IN_MEMORY: [&str; 4] = [
        "sqlite::memory:",
        "sqlite://:memory:",
        "sqlite:",
        "sqlite://",
    ];

    URL_IN_MEMORY.contains(&url)
}

pub async fn parse_job_family_watchers_config_file() -> anyhow::Result<()> {
    if let Ok(job_family_watchers_config_file) = env::var("JOB_FAMILY_WATCHERS_CONFIG_FILE") {
        log::info!("Attempting to read job family watchers config file: {}", job_family_watchers_config_file);

        let content = read_to_string(&job_family_watchers_config_file).map_err(|err| {
            log::warn!("Failed to read config file: {}. Error: {}", job_family_watchers_config_file, err);
            anyhow::anyhow!("Failed to read config file: {}", err)
        })?;

        log::info!("Successfully read config file. Parsing YAML content...");

        let roots = YamlLoader::load_from_str(&content).map_err(|err| {
            log::error!("Failed to parse YAML from config file. Error: {}", err);
            anyhow::anyhow!("Failed to parse YAML: {}", err)
        })?;

        let mut job_family_watchers: Vec<JobFamilyWatcher> = Vec::with_capacity(10);
        for root in roots {
            for object in root {
                let job_family_watcher = JobFamilyWatcher::try_from(object).map_err(|err| {
                    log::error!("Failed to convert object to JobFamilyWatcher. Error: {}", err);
                    anyhow::anyhow!("Failed to convert object to JobFamilyWatcher: {}", err)
                })?;
                job_family_watchers.push(job_family_watcher);
            }
        }

        if !job_family_watchers.is_empty() {
            log::info!("Creating job family watchers in the service... TOT: {}", job_family_watchers.len());
        } else {
            log::info!("No job family watchers to create.");
        }

        stream::iter(job_family_watchers.into_iter())
            .for_each(|job_family_watcher| {
                async move {
                    if let Err(err) = service::job_family_watcher::create_job_family_watcher(job_family_watcher.clone()).await {
                        log::error!("Failed to create job family watcher: {:?}. Error: {}", job_family_watcher, err);
                    }
                }
            })
            .await;
    } else {
        log::warn!("Environment variable JOB_FAMILY_WATCHERS_CONFIG_FILE is not set.");
    }

    Ok(())
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