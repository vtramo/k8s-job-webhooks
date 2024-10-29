use std::fmt;

use chrono::{DateTime, Utc};
use k8s_openapi::serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::models::service;
use crate::models::service::{CreateJobDoneTriggerWebhookRequest, CreateJobDoneTriggerWebhookRequestError, CreateJobDoneWatcherRequest, CreateWebhookRequestError, JobDoneTriggerWebhook, JobDoneTriggerWebhookStatus, JobDoneWatcher, JobDoneWatcherStatus, Webhook};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWebhookRequestApi {
    pub url: String,
    pub request_body: String,
    pub description: String,
}

impl TryFrom<CreateWebhookRequestApi> for service::CreateWebhookRequest {
    type Error = CreateWebhookRequestError;

    fn try_from(create_webhook_request_api: CreateWebhookRequestApi) -> Result<Self, Self::Error> {
        service::CreateWebhookRequest::new(
            &create_webhook_request_api.url,
            &create_webhook_request_api.request_body,
            &create_webhook_request_api.description,
        )
    }
}


#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookApi {
    pub id: String,
    pub url: String,
    pub request_body: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

impl From<&Webhook> for WebhookApi {
    fn from(webhook: &Webhook) -> Self {
        Self {
            id: webhook.id().to_string(),
            url: webhook.url().to_string(),
            request_body: webhook.request_body().to_string(),
            description: webhook.description().to_string(),
            created_at: webhook.created_at(),
        }
    }
}


#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateJobDoneWatcherRequestApi {
    pub job_name: String,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u32,
    pub job_done_trigger_webhooks: Vec<CreateJobDoneTriggerWebhookRequestApi>,
}

impl TryFrom<CreateJobDoneWatcherRequestApi> for CreateJobDoneWatcherRequest {
    type Error = anyhow::Error;

    fn try_from(value: CreateJobDoneWatcherRequestApi) -> Result<Self, Self::Error> {
        let mut webhooks = Vec::with_capacity(value.job_done_trigger_webhooks.len());
        for webhook in value.job_done_trigger_webhooks {
            webhooks.push(CreateJobDoneTriggerWebhookRequest::try_from(webhook)?);
        }
        CreateJobDoneWatcherRequest::new(&value.job_name, value.timeout_seconds, webhooks)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateJobDoneTriggerWebhookRequestApi {
    pub webhook_id: String,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u32,
}

impl TryFrom<CreateJobDoneTriggerWebhookRequestApi> for CreateJobDoneTriggerWebhookRequest {
    type Error = CreateJobDoneTriggerWebhookRequestError;

    fn try_from(value: CreateJobDoneTriggerWebhookRequestApi) -> Result<Self, Self::Error> {
        CreateJobDoneTriggerWebhookRequest::new(
            &value.webhook_id,
            value.timeout_seconds
        )
    }
}


#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JobDoneTriggerWebhookApi {
    pub id: Uuid,
    pub webhook_id: Uuid,
    #[serde(skip_serializing_if = "is_zero")]
    pub timeout_seconds: u32,
    pub status: JobDoneTriggerWebhookStatusApi,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub called_at: Option<DateTime<Utc>>,
}

impl From<JobDoneTriggerWebhook> for JobDoneTriggerWebhookApi {
    fn from(job_done_trigger_webhook: JobDoneTriggerWebhook) -> Self {
        Self {
            id: job_done_trigger_webhook.id(),
            webhook_id: job_done_trigger_webhook.webhook_id(),
            timeout_seconds: job_done_trigger_webhook.timeout_seconds(),
            status: JobDoneTriggerWebhookStatusApi::from(*job_done_trigger_webhook.status()),
            called_at: job_done_trigger_webhook.called_at(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobDoneTriggerWebhookStatusApi {
    Called,
    NotCalled,
    Failed,
    Timeout,
    Cancelled,
}

impl fmt::Display for JobDoneTriggerWebhookStatusApi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = match self {
            JobDoneTriggerWebhookStatusApi::Called => "Called",
            JobDoneTriggerWebhookStatusApi::NotCalled => "NotCalled",
            JobDoneTriggerWebhookStatusApi::Failed => "Failed",
            JobDoneTriggerWebhookStatusApi::Timeout => "Timeout",
            JobDoneTriggerWebhookStatusApi::Cancelled => "Cancelled",
        };
        write!(f, "{}", output)
    }
}

impl From<JobDoneTriggerWebhookStatus> for JobDoneTriggerWebhookStatusApi {
    fn from(value: JobDoneTriggerWebhookStatus) -> Self {
        match value {
            JobDoneTriggerWebhookStatus::Called => JobDoneTriggerWebhookStatusApi::Called,
            JobDoneTriggerWebhookStatus::NotCalled => JobDoneTriggerWebhookStatusApi::NotCalled,
            JobDoneTriggerWebhookStatus::Failed => JobDoneTriggerWebhookStatusApi::Failed,
            JobDoneTriggerWebhookStatus::Timeout => JobDoneTriggerWebhookStatusApi::Timeout,
            JobDoneTriggerWebhookStatus::Cancelled => JobDoneTriggerWebhookStatusApi::Cancelled,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JobDoneWatcherApi {
    pub id: Uuid,
    pub job_name: String,
    #[serde(skip_serializing_if = "is_zero")]
    pub timeout_seconds: u32,
    pub status: JobDoneWatcherStatusApi,
    pub created_at: DateTime<Utc>,
    pub job_done_trigger_webhooks: Vec<JobDoneTriggerWebhookApi>,
}

impl From<JobDoneWatcher> for JobDoneWatcherApi {
    fn from(job_done_watcher: JobDoneWatcher) -> Self {
        JobDoneWatcherApi {
            id: job_done_watcher.id(),
            job_name: job_done_watcher.job_name().to_string(),
            timeout_seconds: job_done_watcher.timeout_seconds(),
            status: JobDoneWatcherStatusApi::from(job_done_watcher.status()),
            created_at: job_done_watcher.created_at(),
            job_done_trigger_webhooks: job_done_watcher
                .job_done_trigger_webhooks()
                .clone()
                .into_iter()
                .map(JobDoneTriggerWebhookApi::from)
                .collect(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(PartialEq)]
pub enum JobDoneWatcherStatusApi {
    Completed,
    PartiallyCompleted,
    Pending,
    Processing,
    Cancelled,
    Failed,
    Timeout,
}

impl From<JobDoneWatcherStatus> for JobDoneWatcherStatusApi {
    fn from(value: JobDoneWatcherStatus) -> Self {
        match value {
            JobDoneWatcherStatus::Completed => JobDoneWatcherStatusApi::Completed,
            JobDoneWatcherStatus::PartiallyCompleted => JobDoneWatcherStatusApi::PartiallyCompleted,
            JobDoneWatcherStatus::Pending => JobDoneWatcherStatusApi::Pending,
            JobDoneWatcherStatus::Processing => JobDoneWatcherStatusApi::Processing,
            JobDoneWatcherStatus::Cancelled => JobDoneWatcherStatusApi::Cancelled,
            JobDoneWatcherStatus::Failed => JobDoneWatcherStatusApi::Failed,
            JobDoneWatcherStatus::Timeout => JobDoneWatcherStatusApi::Timeout,
        }
    }
}


fn is_zero(value: &u32) -> bool {
    *value == 0
}

fn default_timeout_seconds() -> u32 {
    0
}
