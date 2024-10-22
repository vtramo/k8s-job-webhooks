use futures_util::{stream, StreamExt};
use reqwest::Client;

use crate::models::{JobFamilyWatcher, Url};
use crate::repository;


pub async fn create_job_family_watcher(job_family_watcher: JobFamilyWatcher) -> anyhow::Result<()> {
    log::info!("Creating job family watcher (job family {})", job_family_watcher.job_family);

    let job_family_watcher_repository = repository::get_job_family_watcher_repository();
    job_family_watcher_repository.create_job_family_watcher(job_family_watcher).await?;
    Ok(())
}

pub async fn notify_job_family_watchers(job_family: &str) {
    log::info!("Notifying job family watchers for job family: {}", job_family);

    let job_family_watcher_repository = repository::get_job_family_watcher_repository();
    let job_family_watchers =
        match job_family_watcher_repository.find_all_job_family_watchers_by_job_family(job_family).await {
            Ok(job_family_watcher) => job_family_watcher,
            Err(err) => {
                log::error!("Failed to retrieve job family watchers for job family '{}': {:?}", job_family, err);
                return;
            }
        };

    log::info!("Found {} job family watchers for job family: {}", job_family_watchers.len(), job_family);

    stream::iter(job_family_watchers.into_iter())
        .for_each(|job_family_watcher|
            call_webhook(
                job_family_watcher.url,
                job_family_watcher.request_body,
                job_family
            )).await;
}

async fn call_webhook(url: Url, request_body: String, job_family: &str) {
    log::info!("Calling webhook for job family '{}' at URL: {}", job_family, url);

    let http_client = Client::new();
    match http_client.post(url.to_string()).body(request_body).send().await {
        Ok(response) => {
            log::info!("Successfully called webhook at {} with status: {}", url, response.status());
            log::info!("Successfully called webhook");
        },
        Err(err) => {
            log::warn!("Failed to call webhook for job family '{}': {}, URL: {}", job_family, err, url);
        }
    }
}