use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use chrono::Utc;
use uuid::Uuid;

use crate::models::{CreateJobDoneWatcherRequest, JobDoneTriggerWebhook, JobDoneTriggerWebhookStatus, JobDoneWatcher, JobDoneWatcherStatus};
use crate::service::webhooks::get_webhooks_by_id;

pub static JOB_DONE_WATCHERS:
    LazyLock<Mutex<HashMap<String, JobDoneWatcher>>> =
        LazyLock::new(|| {
            Mutex::new(HashMap::with_capacity(10))
        });

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

    match JOB_DONE_WATCHERS.lock() {
        Ok(mut job_done_watchers) => job_done_watchers.insert(job_done_watcher.id.clone(), job_done_watcher.clone()),
        Err(_) => panic!(), // TODO:
    };

    match ACTIVE_JOB_DONE_WATCHERS_IDS_BY_JOB_NAME.lock() {
        Ok(mut job_done_watchers_ids_by_job_name) =>
            job_done_watchers_ids_by_job_name
                .entry(job_done_watcher.job_name.clone())
                .or_insert(Vec::with_capacity(10))
                .push(job_done_watcher.id.clone()),
        Err(_) => panic!(), // TODO:
    };

    job_done_watcher
}

pub async fn get_job_done_watchers() -> Vec<JobDoneWatcher> {
    match JOB_DONE_WATCHERS.lock() {
        Ok(job_done_watchers) => job_done_watchers.values().cloned().collect(),
        Err(_) => panic!(), // TODO:
    }
}

pub async fn get_job_done_watcher_by_id(id: &str) -> Option<JobDoneWatcher> {
    match JOB_DONE_WATCHERS.lock() {
        Ok(job_done_watchers) => job_done_watchers.get(id).cloned(),
        Err(_) => panic!(), // TODO:
    }
}

pub fn is_job_watched(job_name: &str) -> bool {
    match ACTIVE_JOB_DONE_WATCHERS_IDS_BY_JOB_NAME.lock() {
        Ok(active_job_done_watchers_ids_by_job_name) => active_job_done_watchers_ids_by_job_name.contains_key(job_name),
        Err(_) => false,
    }
}

pub async fn notify_job_done_watchers(job_name: &str) {
    match ACTIVE_JOB_DONE_WATCHERS_IDS_BY_JOB_NAME.lock() {
        Ok(mut active_job_done_watchers_ids_by_job_name) =>
            match active_job_done_watchers_ids_by_job_name.get(job_name) {
                None => return,
                Some(active_job_done_watchers_ids) => {
                    let call_webhook_tasks: Vec<_> = active_job_done_watchers_ids
                        .iter()
                        .cloned()
                        .map(|active_job_done_watcher_id| {
                            println!("spawn");
                            actix_web::rt::spawn(async move {
                                call_job_done_trigger_webhooks(&active_job_done_watcher_id).await;
                            })
                        })
                        .collect();

                    for call_webhook_task in call_webhook_tasks {
                        let _ = call_webhook_task.await;
                    }


                    active_job_done_watchers_ids_by_job_name.remove(job_name.clone());
                },
            },
        Err(_) => panic!(), // TODO:
    };
}

async fn call_job_done_trigger_webhooks(job_watcher_id: &str) {
    match JOB_DONE_WATCHERS.lock() {
        Ok(mut job_done_watchers) => match job_done_watchers.get_mut(job_watcher_id) {
            None => return,
            Some(job_done_watcher) => {
                let http_client = reqwest::Client::new();
                println!("calling webhooks");

                let job_done_trigger_webhooks = &mut job_done_watcher.job_done_trigger_webhooks;
                let tot_job_done_trigger_webhooks = job_done_trigger_webhooks.len();
                let mut tot_called_webhooks = 0;
                let mut tot_failed_webhooks = 0;
                for job_done_trigger_webhook in job_done_trigger_webhooks {
                    let webhook = get_webhooks_by_id(&job_done_trigger_webhook.webhook_id)
                        .await
                        .expect("Should be present!");

                    job_done_trigger_webhook.set_called_at(Utc::now());
                    let post_result = http_client.post(webhook.url.to_string())
                        .body(webhook.request_body)
                        .send()
                        .await;

                    match post_result {
                        Ok(_) => {
                            job_done_trigger_webhook.set_status(JobDoneTriggerWebhookStatus::Called);
                            tot_called_webhooks += 1;
                        },
                        Err(_) => {
                            job_done_trigger_webhook.set_status(JobDoneTriggerWebhookStatus::Failed);
                            tot_failed_webhooks += 1;
                        }
                    };
                }

                job_done_watcher.set_status(
                    if tot_job_done_trigger_webhooks == tot_called_webhooks {
                        JobDoneWatcherStatus::Completed
                    } else if tot_job_done_trigger_webhooks == tot_failed_webhooks {
                        JobDoneWatcherStatus::Failed
                    } else {
                        JobDoneWatcherStatus::PartiallyCompleted
                    }
                );
            }
        }
        Err(_) => panic!(), // TODO:
    };
}