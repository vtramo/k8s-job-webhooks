use k8s_job_webhooks::{service, setup};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    setup::init_logging()?;
    setup::init_database().await?;
    service::k8s_job_watcher::spawn_k8s_job_watcher();
    setup::init_http_server().await?;
    setup::parse_job_family_watchers_config_file().await?;
    Ok(())
}