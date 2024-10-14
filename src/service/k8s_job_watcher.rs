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

pub async fn watch_jobs() {
    let client = Client::try_default().await.unwrap();
    let jobs: Api<Job> = Api::default_namespaced(client);
    let stream = watcher(jobs.clone(), watcher::Config::default()).default_backoff().applied_objects();
    pin_mut!(stream);

    while let Some(job) = stream.try_next().await.unwrap() {
        if is_already_scanned_job(job.labels()) {
            println!("Job {:?} already scanned.", job.name());
            continue;
        }

        if let Some((job_name, job_status)) = job.name().zip(job.clone().status) {
            if !service::job_done_watchers::is_job_watched(&job_name) {
                println!("Job {:?} !is_job_watched", job_name);
                continue;
            }
            if !is_successfully_completed_job(job_status) {
                println!("Job {:?} !is_successfully_completed_job", job_name);
                continue;
            }

            println!("Job {:?} notify_job_done_watchers", job_name);
            service::job_done_watchers::notify_job_done_watchers(&job_name).await;
            println!("Job {:?} notify_job_done_watchers exit", job_name);

            println!("{:#?}", add_webhooks_called_label(&jobs, &job_name).await);
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
