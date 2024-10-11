use std::collections::BTreeMap;

use futures_util::{pin_mut, TryStreamExt};
use k8s_openapi::api::batch::v1::{Job, JobStatus};
use k8s_openapi::serde_json::json;
use kube::{Api, Client, ResourceExt};
use kube::api::{Patch, PatchParams};
use kube::runtime::{watcher, WatchStreamExt};
use kube::runtime::reflector::Lookup;

use crate::models::WebHook;
use crate::monitors;

const K8S_WEBHOOKS_CALLED_LABEL: &'static str = "app.k8s.job.monitor/webhooks-called";

pub async fn watch_jobs() {
    let client = Client::try_default().await.unwrap();
    let jobs: Api<Job> = Api::default_namespaced(client);
    let stream = watcher(jobs.clone(), watcher::Config::default()).default_backoff().applied_objects();
    pin_mut!(stream);

    while let Some(event) = stream.try_next().await.unwrap() {
        let is_already_scanned_job = event.name()
            .map(|job_name| is_already_scanned_job(event.labels(), &job_name))
            .unwrap_or(true);
        if is_already_scanned_job { continue; }

        if let Some((job_name, job_status)) = event.name().zip(event.clone().status) {
            if !is_successfully_completed_job(job_status) { continue; }

            let webhooks = get_webhooks_by_job_name(&job_name);
            if webhooks.is_empty() { continue; }

            monitors::call_webhooks(webhooks).await;
            println!("{:#?}", add_webhooks_called_label(&jobs, &job_name).await);
        }
    }
}

async fn add_webhooks_called_label(job_api: &Api<Job>, job_name: &str) -> Result<Job, kube::Error> {
    let patch = json!({
        "metadata": {
            "labels": {
                K8S_WEBHOOKS_CALLED_LABEL: "true"
            }
        }
    });

    let patch_params = PatchParams::default();
    job_api.patch(job_name, &patch_params, &Patch::Merge(&patch)).await
}

fn is_already_scanned_job(job_labels: &BTreeMap<String, String>, job_name: &str) -> bool {
    let is_already_scanned_job = job_labels
        .get(K8S_WEBHOOKS_CALLED_LABEL)
        .map_or(false, |scanned_label| scanned_label == "true");

    println!("is already scanned job {} jobname {}", is_already_scanned_job, job_name);
    is_already_scanned_job
}

fn is_successfully_completed_job(job_status: JobStatus) -> bool {
    let conditions = job_status.conditions;
    println!("{:#?}", conditions);

    const JOB_CONDITION_STATUS_TRUE: &'static str = "True";
    const JOB_CONDITION_TYPE_COMPLETE: &'static str = "Complete";
    conditions
        .and_then(|job_conditions| job_conditions.last().cloned())
        .map(|last_job_condition|
            last_job_condition.status == JOB_CONDITION_STATUS_TRUE &&
            last_job_condition.type_  == JOB_CONDITION_TYPE_COMPLETE)
        .unwrap_or(false)
}

fn get_webhooks_by_job_name(job_name: &str) -> Vec<WebHook> {
    let mut webhooks: Vec<WebHook> = Vec::with_capacity(10);
    println!("Getting webhooks for jobname {}", job_name);

    if let Some(job_webhooks) = monitors::get_webhooks_by_job(job_name) {
        webhooks.extend(job_webhooks);
    }

    if let Some(job_name) = extract_cronjob_name_from_job_name(job_name)
        .and_then(|name| monitors::get_webhooks_by_cronjob(&name)) {
        webhooks.extend(job_name);
    }

    println!("webhooks jobname {}, {:#?}", job_name, webhooks);
    webhooks
}

fn extract_cronjob_name_from_job_name(job_name: &str) -> Option<String> {
    let parts: Vec<&str> = job_name.split('-').collect();

    if parts.len() > 1 {
        Some(parts[0..parts.len() - 1].join("-"))
    } else {
        None
    }
}