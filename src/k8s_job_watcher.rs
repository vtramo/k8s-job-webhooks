use std::collections::BTreeMap;
use futures_util::{pin_mut, TryStreamExt};
use k8s_openapi::api::batch::v1::{Job, JobSpec, JobStatus};
use k8s_openapi::serde_json::json;
use kube::{Api, Client};
use kube::api::{Patch, PatchParams};
use kube::runtime::{watcher, WatchStreamExt};
use crate::monitors;
use crate::models::WebHook;

static K8S_WEBHOOKS_CALLED_LABEL: &'static str = "app.k8s.job.monitor/webhooks-called";

pub async fn watch_jobs() {
    let client = Client::try_default().await.unwrap();
    let jobs: Api<Job> = Api::default_namespaced(client);
    let mut stream = watcher(jobs.clone(), watcher::Config::default()).default_backoff().applied_objects();
    pin_mut!(stream);

    while let Some(event) = stream.try_next().await.unwrap() {
        let is_already_scanned_job = event.clone().metadata.labels
            .map(|job_labels| is_already_scanned_job(&job_labels, &event.clone().metadata.name.unwrap()))
            .unwrap_or(true);
        if is_already_scanned_job { continue; }

        if let Some(((job_name, job_status), job_spec)) = event.metadata.name.zip(event.status).zip(event.spec) {
            if !is_successfully_completed_job(&job_status, &job_spec, &job_name) { continue; }

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

fn is_successfully_completed_job(job_status: &JobStatus, job_spec: &JobSpec, job_name: &str) -> bool {
    let succeeded = job_status.succeeded.unwrap_or(0);
    let completion = job_spec.completions.unwrap_or(-1);
    let completed_successfully = succeeded == completion;
    println!("completed_successfully {}, job_name: {}", completed_successfully, job_name);
    completed_successfully
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