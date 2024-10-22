use k8s_job_webhooks::service::k8s_job_watcher;
use k8s_job_webhooks::setup;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    setup::init_logging()?;
    setup::init_database().await?;
    k8s_job_watcher::spawn_k8s_job_watcher();
    setup::init_http_server().await?;

    Ok(())
}