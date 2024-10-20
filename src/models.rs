use std::fmt::Display;

use chrono::{DateTime, Utc};
use k8s_openapi::serde::Deserialize;
use serde::{Deserializer, Serialize, Serializer};

use crate::models::entity::{JobDoneTriggerWebhookEntity, JobDoneTriggerWebhookStatusEntity, JobDoneWatcherEntity, JobDoneWatcherStatusEntity, WebhookEntity};

pub mod entity;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateWebhookRequest {
    pub url: Url,
    pub request_body: String,
    pub description: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    pub id: String,
    pub url: Url,
    pub request_body: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

impl From<&WebhookEntity> for Webhook {
    fn from(webhook_entity: &WebhookEntity) -> Self {
        Self {
            id: webhook_entity.id.clone(),
            url: Url(url::Url::parse(&webhook_entity.url).expect("url should be correct!")),
            request_body: webhook_entity.request_body.clone(),
            description: webhook_entity.description.clone(),
            created_at: webhook_entity.created_at.and_utc(),
        }
    }
}

impl From<WebhookEntity> for Webhook {
    fn from(webhook_entity: WebhookEntity) -> Self {
        Self {
            id: webhook_entity.id,
            url: Url(url::Url::parse(&webhook_entity.url).expect("url should be correct!")),
            request_body: webhook_entity.request_body,
            description: webhook_entity.description,
            created_at: webhook_entity.created_at.and_utc(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JobDoneTriggerWebhook {
    pub id: String,
    pub webhook_id: String,
    #[serde(skip_serializing_if = "is_zero")]
    pub timeout_seconds: u32,
    pub status: JobDoneTriggerWebhookStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub called_at: Option<DateTime<Utc>>,
}

impl JobDoneTriggerWebhook {
    pub fn set_called_at(&mut self, date_time: DateTime<Utc>) {
        self.called_at = Some(date_time);
    }

    pub fn set_status(&mut self, status: JobDoneTriggerWebhookStatus) {
        self.status = status;
    }
}

impl From<&JobDoneTriggerWebhookEntity> for JobDoneTriggerWebhook {
    fn from(job_done_trigger_webhook_entity: &JobDoneTriggerWebhookEntity) -> Self {
        Self {
            id: job_done_trigger_webhook_entity.id.clone(),
            webhook_id: job_done_trigger_webhook_entity.webhook_id.clone(),
            timeout_seconds: job_done_trigger_webhook_entity.timeout_seconds as u32,
            status: (&job_done_trigger_webhook_entity.status).into(),
            called_at: job_done_trigger_webhook_entity.called_at.map(|naive_date_time| naive_date_time.and_utc()),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobDoneTriggerWebhookStatus {
    Called,
    NotCalled,
    Failed,
    Timeout,
    Cancelled,
}

impl From<&JobDoneTriggerWebhookStatusEntity> for JobDoneTriggerWebhookStatus {
    fn from(job_done_trigger_webhook_status_entity: &JobDoneTriggerWebhookStatusEntity) -> Self {
        match job_done_trigger_webhook_status_entity {
            JobDoneTriggerWebhookStatusEntity::Called => JobDoneTriggerWebhookStatus::Called,
            JobDoneTriggerWebhookStatusEntity::NotCalled => JobDoneTriggerWebhookStatus::NotCalled,
            JobDoneTriggerWebhookStatusEntity::Failed => JobDoneTriggerWebhookStatus::Failed,
            JobDoneTriggerWebhookStatusEntity::Timeout => JobDoneTriggerWebhookStatus::Timeout,
            JobDoneTriggerWebhookStatusEntity::Cancelled => JobDoneTriggerWebhookStatus::Cancelled,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateJobDoneWatcherRequest {
    pub job_name: String,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u32,
    pub job_done_trigger_webhooks: Vec<CreateJobDoneTriggerWebhookRequest>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateJobDoneTriggerWebhookRequest {
    pub webhook_id: String,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u32,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JobDoneWatcher {
    pub id: String,
    pub job_name: String,
    #[serde(skip_serializing_if = "is_zero")]
    pub timeout_seconds: u32,
    pub status: JobDoneWatcherStatus,
    pub created_at: DateTime<Utc>,
    pub job_done_trigger_webhooks: Vec<JobDoneTriggerWebhook>,
}

impl JobDoneWatcher {
    pub fn set_status(&mut self, status: JobDoneWatcherStatus) {
        self.status = status;
    }
}

impl From<JobDoneWatcherEntity> for JobDoneWatcher {
    fn from(job_done_watcher_entity: JobDoneWatcherEntity) -> Self {
        Self {
            id: job_done_watcher_entity.id,
            job_name: job_done_watcher_entity.job_name,
            timeout_seconds: job_done_watcher_entity.timeout_seconds as u32,
            status: job_done_watcher_entity.status.into(),
            created_at: job_done_watcher_entity.created_at.and_utc(),
            job_done_trigger_webhooks: job_done_watcher_entity.job_done_trigger_webhooks.iter().map(JobDoneTriggerWebhook::from).collect(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(PartialEq)]
pub enum JobDoneWatcherStatus {
    Completed,
    PartiallyCompleted,
    Pending,
    Cancelled,
    Failed,
    Timeout,
}

impl From<JobDoneWatcherStatusEntity> for JobDoneWatcherStatus {
    fn from(job_done_watcher_status: JobDoneWatcherStatusEntity) -> Self {
        match job_done_watcher_status {
            JobDoneWatcherStatusEntity::Completed => JobDoneWatcherStatus::Completed,
            JobDoneWatcherStatusEntity::PartiallyCompleted => JobDoneWatcherStatus::PartiallyCompleted,
            JobDoneWatcherStatusEntity::Pending => JobDoneWatcherStatus::Pending,
            JobDoneWatcherStatusEntity::Cancelled => JobDoneWatcherStatus::Cancelled,
            JobDoneWatcherStatusEntity::Failed => JobDoneWatcherStatus::Failed,
            JobDoneWatcherStatusEntity::Timeout => JobDoneWatcherStatus::Timeout,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Url(url::Url);

impl Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl Serialize for Url {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

impl<'de> Deserialize<'de> for Url {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?; // TODO:
        Ok(Url(url::Url::parse(s).map_err(serde::de::Error::custom)?))
    }
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

fn default_timeout_seconds() -> u32 {
    0
}
