use std::fmt;
use std::fmt::Display;

use chrono::{DateTime, Utc};
use k8s_openapi::serde::Deserialize;
use serde::{Deserializer, Serialize, Serializer};
use uuid::Uuid;
use yaml_rust2::Yaml;

use crate::models::entity::{JobDoneTriggerWebhookEntity, JobDoneTriggerWebhookStatusEntity, JobDoneWatcherEntity, JobDoneWatcherStatusEntity, JobFamilyWatcherEntity, WebhookEntity};

pub mod entity;
pub mod service;

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
    pub id: Uuid,
    pub url: Url,
    pub request_body: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

impl From<&WebhookEntity> for Webhook {
    fn from(webhook_entity: &WebhookEntity) -> Self {
        Self {
            id: Uuid::parse_str(&webhook_entity.id).expect("Uuid from db should be correct!"),
            url: Url(url::Url::parse(&webhook_entity.url).expect("url should be correct!")),
            request_body: webhook_entity.request_body.clone(),
            description: webhook_entity.description.clone(),
            created_at: webhook_entity.created_at,
        }
    }
}

impl From<WebhookEntity> for Webhook {
    fn from(webhook_entity: WebhookEntity) -> Self {
        Self {
            id: Uuid::parse_str(&webhook_entity.id).expect("Uuid from db should be correct!"),
            url: Url(url::Url::parse(&webhook_entity.url).expect("url should be correct!")),
            request_body: webhook_entity.request_body,
            description: webhook_entity.description,
            created_at: webhook_entity.created_at,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JobDoneTriggerWebhook {
    pub id: Uuid,
    pub webhook_id: Uuid,
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
            id: Uuid::parse_str(&job_done_trigger_webhook_entity.id).expect("Uuid from db should be correct!"),
            webhook_id: Uuid::parse_str(&job_done_trigger_webhook_entity.webhook_id).expect("Uuid from db should be correct!"),
            timeout_seconds: job_done_trigger_webhook_entity.timeout_seconds as u32,
            status: (&job_done_trigger_webhook_entity.status).into(),
            called_at: job_done_trigger_webhook_entity.called_at.map(|naive_date_time| naive_date_time),
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

impl fmt::Display for JobDoneTriggerWebhookStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = match self {
            JobDoneTriggerWebhookStatus::Called => "Called",
            JobDoneTriggerWebhookStatus::NotCalled => "NotCalled",
            JobDoneTriggerWebhookStatus::Failed => "Failed",
            JobDoneTriggerWebhookStatus::Timeout => "Timeout",
            JobDoneTriggerWebhookStatus::Cancelled => "Cancelled",
        };
        write!(f, "{}", output)
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
    pub webhook_id: Uuid,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u32,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JobDoneWatcher {
    pub id: Uuid,
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
            id: Uuid::parse_str(&job_done_watcher_entity.id).expect("Uuid from db should be correct!"),
            job_name: job_done_watcher_entity.job_name,
            timeout_seconds: job_done_watcher_entity.timeout_seconds as u32,
            status: job_done_watcher_entity.status.into(),
            created_at: job_done_watcher_entity.created_at,
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
    Processing,
    Cancelled,
    Failed,
    Timeout,
}

impl fmt::Display for JobDoneWatcherStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_str = match self {
            JobDoneWatcherStatus::Completed => "Completed",
            JobDoneWatcherStatus::PartiallyCompleted => "PartiallyCompleted",
            JobDoneWatcherStatus::Pending => "Pending",
            JobDoneWatcherStatus::Cancelled => "Cancelled",
            JobDoneWatcherStatus::Failed => "Failed",
            JobDoneWatcherStatus::Timeout => "Timeout",
            JobDoneWatcherStatus::Processing => "Processing",
        };
        write!(f, "{}", status_str)
    }
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
            JobDoneWatcherStatusEntity::Processing => JobDoneWatcherStatus::Processing,
        }
    }
}

#[derive(Clone, Debug)]
pub struct JobFamilyWatcher {
    pub job_family: String,
    pub url: Url,
    pub request_body: String,
    pub description: String
}

impl From<JobFamilyWatcherEntity> for JobFamilyWatcher {
    fn from(job_family_watcher_entity: JobFamilyWatcherEntity) -> Self {
        JobFamilyWatcher {
            job_family: job_family_watcher_entity.job_family,
            url: Url::new(&job_family_watcher_entity.url).expect("Should be correct!"),
            request_body: job_family_watcher_entity.request_body,
            description: job_family_watcher_entity.description,
        }
    }
}

impl TryFrom<Yaml> for JobFamilyWatcher {
    type Error = anyhow::Error;

    fn try_from(yaml: Yaml) -> Result<Self, Self::Error> {
        let job_family = extract_yaml_string(&yaml, "jobFamily")?;
        let url = Url::new(&extract_yaml_string(&yaml, "url")?)?;
        let request_body = extract_yaml_string(&yaml, "requestBody").unwrap_or_default();
        let description = extract_yaml_string(&yaml, "description").unwrap_or_default();

        Ok(Self {
            job_family,
            url,
            request_body,
            description,
        })
    }
}

// Helper function to extract a string from Yaml with error handling
fn extract_yaml_string(yaml: &Yaml, key: &str) -> Result<String, anyhow::Error> {
    match &yaml[key] {
        Yaml::String(value) => Ok(value.clone()),
        _ => Err(anyhow::anyhow!("Missing or invalid value for key: {}", key)),
    }
}


#[derive(Debug, Clone)]
pub struct Url(url::Url);

impl Url {
    pub fn new(url: &str) -> anyhow::Result<Self> {
        Ok(Url(url::Url::parse(url)?))
    }
}

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
        let s: &str = Deserialize::deserialize(deserializer)?;
        Ok(Url(url::Url::parse(s).map_err(serde::de::Error::custom)?))
    }
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

fn default_timeout_seconds() -> u32 {
    0
}
