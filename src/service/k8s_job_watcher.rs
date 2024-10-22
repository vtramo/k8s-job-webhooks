use std::collections::BTreeMap;

use futures_util::{pin_mut, TryStreamExt};
use k8s_openapi::api::batch::v1::{Job, JobStatus};
use k8s_openapi::serde_json::json;
use kube::{Api, Client, ResourceExt};
use kube::api::{Patch, PatchParams};
use kube::runtime::{watcher, WatchStreamExt};
use kube::runtime::reflector::Lookup;

use crate::service;

const K8S_WEBHOOKS_CALLED_LABEL: &'static str = "app.k8s.job.webhooks/webhooks-called";

pub fn spawn_k8s_job_watcher() {
    actix_web::rt::spawn(watch_jobs());
}

pub async fn watch_jobs() {
    log::info!("Starting K8S watch jobs...");

    let client = Client::try_default().await.unwrap();
    let jobs: Api<Job> = Api::default_namespaced(client);
    let stream = watcher(jobs.clone(), watcher::Config::default()).default_backoff().applied_objects();
    pin_mut!(stream);

    log::info!("K8S job watcher initialized successfully.");
    while let Some(job) = stream.try_next().await.unwrap() {
        log::debug!("Received job update: {:?}", job.name());

        if is_already_scanned_job(job.labels()) {
            log::debug!("Job {:?} already scanned, skipping.", job.name());
            continue;
        }

        if let Some((job_name, job_status)) = job.name().zip(job.clone().status) {
            log::debug!("Processing job: {}", job_name);

            if !is_successfully_completed_job(job_status) {
                log::info!("Job {} not successfully completed, skipping.", job_name);
                continue;
            }

            log::info!("Job {} successfully completed, notifying watchers...", job_name);
            service::job_done_watchers::notify_job_done_watchers(&job_name).await;

            log::info!("Adding label to indicate webhooks have been called for job: {}", job_name);

            if let Err(err) = add_webhooks_called_label(&jobs, &job_name).await {
                log::warn!("Failed to add webhooks-called label to job {}: {:?}", job_name, err);
            } else {
                log::info!("Label added to job {} successfully.", job_name);
            }
        }
    }
}

fn is_already_scanned_job(job_labels: &BTreeMap<String, String>) -> bool {
    job_labels
        .get(K8S_WEBHOOKS_CALLED_LABEL)
        .map_or(false, |scanned_label| scanned_label == "true")
}

fn is_successfully_completed_job(job_status: JobStatus) -> bool {
    let conditions = job_status.conditions;

    const JOB_CONDITION_STATUS_TRUE: &'static str = "True";
    const JOB_CONDITION_TYPE_COMPLETE: &'static str = "Complete";
    conditions
        .and_then(|job_conditions| job_conditions.last().cloned())
        .map(|last_job_condition|
            last_job_condition.status == JOB_CONDITION_STATUS_TRUE &&
                last_job_condition.type_  == JOB_CONDITION_TYPE_COMPLETE)
        .unwrap_or(false)
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
