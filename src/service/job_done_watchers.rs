use std::time::Duration;

use chrono::Utc;
use futures_util::{stream, StreamExt};
use futures_util::future::join_all;
use reqwest::Client;
use uuid::Uuid;

use crate::{repository, service};
use crate::models::service::{CreateJobDoneWatcherRequest, JobDoneTriggerWebhook, JobDoneTriggerWebhookStatus, JobDoneWatcher, JobDoneWatcherStatus, JobName};

pub async fn create_job_done_watcher(create_job_done_watcher_request: CreateJobDoneWatcherRequest) -> anyhow::Result<JobDoneWatcher> {
    log::info!("Creating JobDoneWatcher for job: {}", create_job_done_watcher_request.job_name());

    let job_done_trigger_webhooks: Vec<_> = create_job_done_watcher_request.job_done_trigger_webhooks()
        .iter()
        .map(|job_done_trigger_webhook| JobDoneTriggerWebhook::new(
            Uuid::new_v4(),
            job_done_trigger_webhook.webhook_id(),
            job_done_trigger_webhook.timeout_seconds(),
            JobDoneTriggerWebhookStatus::NotCalled,
            None,
        )).collect();

    let job_done_watcher = JobDoneWatcher::new(
        Uuid::new_v4(),
        create_job_done_watcher_request.job_name().clone(),
        create_job_done_watcher_request.timeout_seconds(),
        job_done_trigger_webhooks,
        JobDoneWatcherStatus::Pending,
        Utc::now(),
    );

    if job_done_watcher.timeout_seconds() > 0 {
        start_timer_job_done_watcher(&job_done_watcher.id(), job_done_watcher.timeout_seconds() as u64);
    }

    let job_done_watcher_repository = repository::get_job_done_watcher_repository();

    job_done_watcher_repository.create_watcher(&job_done_watcher).await
        .map(|_| {
            log::info!("Successfully created JobDoneWatcher with ID: {}", job_done_watcher.id());
            job_done_watcher
        })
        .map_err(|error| {
            log::error!("Failed to create JobDoneWatcher: {}", error);
            anyhow::anyhow!("Failed to create job_done_watcher: {}", error)
        })
}

fn start_timer_job_done_watcher(job_done_watcher_id: &Uuid, timeout_secs: u64) {
    log::info!("Starting timeout of {} seconds for JobDoneWatcher with ID: {}", timeout_secs, job_done_watcher_id);

    let job_done_watcher_id = job_done_watcher_id.clone();
    actix_web::rt::spawn(async move {
        actix_web::rt::time::sleep(Duration::from_secs(timeout_secs)).await;
        log::info!("Timeout reached for JobDoneWatcher ID: {}", job_done_watcher_id);

        let job_done_watcher_repository = repository::get_job_done_watcher_repository();
        let timeout_result = job_done_watcher_repository.update_watcher_status(&job_done_watcher_id, JobDoneWatcherStatus::Timeout).await;

        match timeout_result {
            Ok(_) => log::info!("JobDoneWatcher {} updated to Timeout status", job_done_watcher_id),
            Err(error) => log::error!("Failed to update JobDoneWatcher {} to Timeout: {:#?}", job_done_watcher_id, error),
        };
    });
}

pub async fn get_job_done_watchers() -> anyhow::Result<Vec<JobDoneWatcher>> {
    log::info!("Fetching all JobDoneWatchers");

    let job_done_watcher_repository = repository::get_job_done_watcher_repository();
    job_done_watcher_repository.find_all_watchers().await
}

pub async fn get_job_done_watcher_by_id(job_done_watcher_id: &Uuid) -> anyhow::Result<Option<JobDoneWatcher>> {
    log::info!("Fetching JobDoneWatcher by ID: {}", job_done_watcher_id);

    let job_done_watcher_repository = repository::get_job_done_watcher_repository();
    job_done_watcher_repository.find_watcher_by_id(job_done_watcher_id).await
}

pub async fn notify_job_done_watchers(job_name: &JobName) {
    log::info!("Notifying JobDoneWatchers for job: {}", job_name);

    let job_done_watcher_repository = repository::get_job_done_watcher_repository();
    let job_done_watchers =
        match job_done_watcher_repository.update_watchers_status_by_job_name_and_status(
            job_name,
            JobDoneWatcherStatus::Pending,
            JobDoneWatcherStatus::Processing
        ).await {
            Ok(updated_job_done_watchers) => updated_job_done_watchers,
            Err(error) => {
                log::error!("Failed to notify JobDoneWatchers for job {}: {:?}", job_name, error);
                return;
            }
        };

    log::info!("Updated status for {} JobDoneWatchers for job: {}", job_done_watchers.len(), job_name);

    stream::iter(job_done_watchers.into_iter())
        .map(|job_done_watcher| call_job_done_trigger_webhooks(job_done_watcher))
        .buffer_unordered(10)
        .collect::<Vec<anyhow::Result<JobDoneWatcher>>>()
        .await
        .iter()
        .for_each(|result| {
            match result {
                Ok(job_done_watcher) => log::info!("JobDoneWatcher {} successfully notified!", job_done_watcher.id()),
                Err(error) => log::error!("Failed to notify JobDoneWatcher: {:#?}", error),
            }
        });
}

async fn call_job_done_trigger_webhooks(mut job_done_watcher: JobDoneWatcher) -> anyhow::Result<JobDoneWatcher> {
    log::info!("Calling webhooks for JobDoneWatcher {}", job_done_watcher.id());

    let job_done_watcher_id = job_done_watcher.id();
    let job_done_trigger_webhooks = job_done_watcher.job_done_trigger_webhooks_mut();
    let total_webhooks = job_done_trigger_webhooks.len();

    let call_webhook_tasks: Vec<_> = job_done_trigger_webhooks
        .iter_mut()
        .map(|webhook| async {
            let result = call_job_done_trigger_webhook(webhook).await;
            let job_done_watcher_repository = repository::get_job_done_watcher_repository();
            job_done_watcher_repository.update_job_done_trigger_webhook_status_and_called_at(
                &job_done_watcher_id,
                &webhook.id(),
                webhook.status().clone(),
                webhook.called_at().expect("Should be not empty"),
            ).await?;
            result
        })
        .collect();

    let call_webhook_results: Vec<_> = join_all(call_webhook_tasks).await;

    let webhooks_sent_successfully: Vec<_> = call_webhook_results
        .into_iter()
        .filter(|result| match result {
            Ok(sent) => *sent,
            Err(error) => {
                log::warn!("Webhook failed: {:#?}", error);
                false
            }
        })
        .collect();

    let total_webhooks_sent_successfully = webhooks_sent_successfully.len();
    let total_webhooks_failed = total_webhooks - total_webhooks_sent_successfully;
    let job_done_watcher_status = evaluate_job_done_watcher_status(
        total_webhooks,
        total_webhooks_sent_successfully,
        total_webhooks_failed
    );

    job_done_watcher.set_status(job_done_watcher_status);
    let job_done_watcher_repository = repository::get_job_done_watcher_repository();
    job_done_watcher_repository.update_watcher_status(&job_done_watcher.id(), job_done_watcher_status).await?;
    log::info!("JobDoneWatcher {} status updated to {:?}", job_done_watcher.id(), job_done_watcher_status);
    Ok(job_done_watcher)
}

async fn call_job_done_trigger_webhook(job_done_trigger_webhook: &mut JobDoneTriggerWebhook) -> anyhow::Result<bool> {
    let webhook_id = job_done_trigger_webhook.webhook_id();
    log::info!("Calling webhook with ID: {}", webhook_id);

    match service::webhooks::get_webhook_by_id(&webhook_id).await? {
        Some(webhook) => {
            job_done_trigger_webhook.set_called_at(Utc::now());
            let http_client = Client::new();
            match http_client
                .post(webhook.url().to_string())
                .body(webhook.request_body().to_string())
                .send().await
            {
                Ok(_) => {
                    job_done_trigger_webhook.set_status(JobDoneTriggerWebhookStatus::Called);
                    log::info!("Successfully called webhook with ID: {}", webhook_id);
                    Ok(true)
                },
                Err(err) => {
                    job_done_trigger_webhook.set_status(JobDoneTriggerWebhookStatus::Failed);
                    log::error!("Failed to call webhook with ID {}: {}", webhook_id, err);
                    Ok(false)
                }
            }
        },
        None => {
            log::warn!("Webhook with ID {} doesn't exist", webhook_id);
            job_done_trigger_webhook.set_status(JobDoneTriggerWebhookStatus::Failed);
            Ok(false)
        }
    }
}

fn evaluate_job_done_watcher_status(total_webhooks: usize, success_count: usize, failure_count: usize) -> JobDoneWatcherStatus {
    match (success_count, failure_count, total_webhooks) {
        (_, 0, _) => JobDoneWatcherStatus::Completed,
        (0, _, _) => JobDoneWatcherStatus::Failed,
        _ => JobDoneWatcherStatus::PartiallyCompleted,
    }
}
