use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use crate::models::{CronJobsMonitor, JobsMonitor, WebHook};

pub static WEBHOOKS_BY_CRONJOB:
    LazyLock<Mutex<HashMap<String, Vec<WebHook>>>> =
        LazyLock::new(|| {
            Mutex::new(HashMap::new())
        });

pub static WEBHOOKS_BY_JOB:
    LazyLock<Mutex<HashMap<String, Vec<WebHook>>>> =
        LazyLock::new(|| {
            Mutex::new(HashMap::new())
        });


pub async fn call_webhooks(webhooks: Vec<WebHook>) {
    let client = reqwest::Client::new();
    println!("calling webhooks");
    for webhook in webhooks {
        let _ = client.post(webhook.get_url())
            .body(webhook.get_request_body())
            .send()
            .await;
    }
}

pub fn add_cronjobs_monitor(cronjobs_monitor: CronJobsMonitor) {
    match WEBHOOKS_BY_CRONJOB.lock() {
        Ok(mut webhooks_by_cronjob) => {
            let cronjobs = cronjobs_monitor.get_cronjobs();
            let webhooks = cronjobs_monitor.get_webhooks();
            for cronjob in cronjobs {
                webhooks_by_cronjob.entry(cronjob)
                    .or_insert(Vec::new())
                    .extend(webhooks.clone());
            }
        },
        Err(error) => { println!("error add cronjob monitor {}", error) }
    }
}

pub fn get_webhooks_by_cronjob(cronjob: &str) -> Option<Vec<WebHook>> {
    match WEBHOOKS_BY_CRONJOB.lock() {
        Ok(webhooks_by_cronjob) => webhooks_by_cronjob.get(cronjob).cloned(),
        Err(error) => { println!("error get_webhook_by_cronjob {}", error); panic!(); }
    }
}

pub fn is_monitored_cronjob(cronjob: &str) -> bool {
    match WEBHOOKS_BY_CRONJOB.lock() {
        Ok(webhooks_by_cronjob) => webhooks_by_cronjob.contains_key(cronjob),
        Err(error) => { println!("error get_webhook_by_cronjob {}", error); panic!(); }
    }
}

pub fn add_jobs_monitor(jobs_monitor: JobsMonitor) {
    match WEBHOOKS_BY_JOB.lock() {
        Ok(mut webhooks_by_job) => {
            let jobs = jobs_monitor.get_jobs();
            let webhooks = jobs_monitor.get_webhooks();
            for job in jobs {
                webhooks_by_job.entry(job)
                    .or_insert(Vec::new())
                    .extend(webhooks.clone());
            }
        }
        Err(error) => { println!("error add_jobs_monitor {}", error) }
    }
}

pub fn is_monitored_job(job: &str) -> bool {
    match WEBHOOKS_BY_JOB.lock() {
        Ok(webhooks_by_job) => webhooks_by_job.contains_key(job),
        Err(error) => { println!("error is_monitored_job {}", error); panic!(); }
    }
}

pub fn get_webhooks_by_job(job: &str) -> Option<Vec<WebHook>> {
    match WEBHOOKS_BY_JOB.lock() {
        Ok(webhooks_by_job) => webhooks_by_job.get(job).cloned(),
        Err(error) => { println!("error get_webhooks_by_job {}", error); panic!(); }
    }
}