use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

use chrono::Utc;
use futures_util::{stream, StreamExt};
use futures_util::future::join_all;
use reqwest::Client;
use uuid::Uuid;

use crate::{repository, service};
use crate::models::{CreateJobDoneWatcherRequest, JobDoneTriggerWebhook, JobDoneTriggerWebhookStatus, JobDoneWatcher, JobDoneWatcherStatus};

pub static ACTIVE_JOB_DONE_WATCHERS_IDS_BY_JOB_NAME:
    LazyLock<Mutex<HashMap<String, Vec<String>>>> =
        LazyLock::new(|| {
            Mutex::new(HashMap::with_capacity(10))
        });


pub async fn create_job_done_watcher(job_done_watcher: CreateJobDoneWatcherRequest) -> JobDoneWatcher {
    let job_done_trigger_webhooks: Vec<_> = job_done_watcher.job_done_trigger_webhooks
        .iter()
        .map(|job_done_trigger_webhook| JobDoneTriggerWebhook {
            id: Uuid::new_v4().to_string(),
            webhook_id: job_done_trigger_webhook.webhook_id.clone(),
            timeout_seconds: job_done_trigger_webhook.timeout_seconds,
            status: JobDoneTriggerWebhookStatus::NotCalled,
            called_at: None,
        }).collect();

    let job_done_watcher = JobDoneWatcher {
        id: Uuid::new_v4().to_string(),
        job_name: job_done_watcher.job_name.clone(),
        timeout_seconds: job_done_watcher.timeout_seconds,
        job_done_trigger_webhooks,
        status: JobDoneWatcherStatus::Pending,
        created_at: Default::default(),
    };

    if job_done_watcher.timeout_seconds > 0 {
        start_timer_job_done_watcher(&job_done_watcher.id, job_done_watcher.timeout_seconds as u64);
    }

    let job_done_watcher_repository = repository::get_job_done_watcher_repository();
    job_done_watcher_repository.save(job_done_watcher.clone()).await;

    job_done_watcher
}

fn start_timer_job_done_watcher(job_done_watcher_id: &str, timeout_secs: u64) {
    let job_done_watcher_id = job_done_watcher_id.to_string();
    println!("Start timeout {} for {}", timeout_secs, job_done_watcher_id);
    actix_web::rt::spawn(async move {
        actix_web::rt::time::sleep(Duration::from_secs(timeout_secs)).await;
        println!("Timeout {} for {}", timeout_secs, job_done_watcher_id);
        let job_done_watcher_repository = repository::get_job_done_watcher_repository();
        job_done_watcher_repository.lock(&job_done_watcher_id,
             Box::new(|mut job_done_watcher| Box::new(async move {
                 if job_done_watcher.status == JobDoneWatcherStatus::Pending {
                     job_done_watcher.status = JobDoneWatcherStatus::Timeout;
                     job_done_watcher.job_done_trigger_webhooks
                         .iter_mut()
                         .for_each(|job_done_trigger_webhook|
                             job_done_trigger_webhook.status = JobDoneTriggerWebhookStatus::Timeout);
                     let job_done_watcher_repository = repository::get_job_done_watcher_repository();
                     job_done_watcher_repository.save(job_done_watcher).await;
                 }
             }))).await;
    });
}


pub async fn get_job_done_watchers() -> Vec<JobDoneWatcher> {
    let job_done_watcher_repository = repository::get_job_done_watcher_repository();
    job_done_watcher_repository.find_all().await
}

pub async fn get_job_done_watcher_by_id(job_done_watcher_id: &str) -> Option<JobDoneWatcher> {
    let job_done_watcher_repository = repository::get_job_done_watcher_repository();
    job_done_watcher_repository.find_by_id(job_done_watcher_id).await
}

pub async fn notify_job_done_watchers(job_name: &str) {
    let pending_job_done_watcher_ids: Vec<_> = find_pending_job_done_watchers(job_name)
        .await
        .iter_mut()
        .map(|job_done_watcher| job_done_watcher.id.clone())
        .collect();

    stream::iter(pending_job_done_watcher_ids.iter())
        .for_each(|active_job_done_watcher_id|
            process_job_done_watcher(active_job_done_watcher_id))
        .await;
}

async fn find_pending_job_done_watchers(job_name: &str) -> Vec<JobDoneWatcher> {
    let job_done_watcher_repository = repository::get_job_done_watcher_repository();

    job_done_watcher_repository.find_all_by_job_name_and_status(
        job_name,
        JobDoneWatcherStatus::Pending).await
}

async fn process_job_done_watcher(job_done_watcher_id: &str) {
    let job_done_watcher_repository = repository::get_job_done_watcher_repository();

    job_done_watcher_repository.lock(job_done_watcher_id,
        Box::new(|job_done_watcher|
            Box::new(call_job_done_trigger_webhooks(job_done_watcher))))
        .await;
}

async fn call_job_done_trigger_webhooks(mut job_done_watcher: JobDoneWatcher) {
    let job_done_trigger_webhooks = &mut job_done_watcher.job_done_trigger_webhooks;
    let total_webhooks = job_done_trigger_webhooks.len();

    let call_webhook_tasks: Vec<_> = job_done_trigger_webhooks
        .iter_mut()
        .map(|webhook| call_job_done_trigger_webhook(webhook))
        .collect();

    let call_webhook_results: Vec<_> = join_all(call_webhook_tasks).await;

    let webhooks_sent_successfully: Vec<_> = call_webhook_results
        .into_iter()
        .filter(|result| *result)
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
    job_done_watcher_repository.save(job_done_watcher).await;
}

async fn call_job_done_trigger_webhook(job_done_trigger_webhook: &mut JobDoneTriggerWebhook) -> bool {
    let webhook_id = job_done_trigger_webhook.webhook_id.clone();
    match service::webhooks::get_webhooks_by_id(&webhook_id).await {
        Some(webhook) => {
            job_done_trigger_webhook.set_called_at(Utc::now());
            let http_client = Client::new();
            match http_client.post(webhook.url.to_string()).body(webhook.request_body).send().await {
                Ok(_) => {
                    job_done_trigger_webhook.set_status(JobDoneTriggerWebhookStatus::Called);
                    true
                },
                Err(err) => {
                    job_done_trigger_webhook.set_status(JobDoneTriggerWebhookStatus::Failed);
                    eprintln!("Failed to call webhook: {}", err);
                    false
                }
            }
        },
        None => {
            eprintln!("Webhook with id {} doesn't exist", webhook_id);
            job_done_trigger_webhook.set_status(JobDoneTriggerWebhookStatus::Failed);
            false
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
