use std::fmt::{Display, Formatter};
use std::ops::Deref;

use chrono::{DateTime, Utc};
use k8s_openapi::serde_json;
use serde::Deserialize;
use uuid::Uuid;

use crate::models::service::{JobDoneTriggerWebhook, JobDoneTriggerWebhookStatus, JobDoneWatcher, JobDoneWatcherStatus, JobFamilyWatcher, JobName, Webhook};

#[derive(sqlx::FromRow, Debug)]
pub struct WebhookEntity {
    pub id: String,
    pub url: String,
    pub request_body: String,
    pub description: String,
    pub created_at: chrono::DateTime<Utc>,
}

impl From<WebhookEntity> for Webhook {
    fn from(webhook_entity: WebhookEntity) -> Self {
        Self::new(
            webhook_entity.id.parse().unwrap(),
            webhook_entity.url.parse().unwrap(),
            webhook_entity.request_body.as_str(),
            webhook_entity.description.as_str(),
            webhook_entity.created_at
        )
    }
}

impl From<&WebhookEntity> for Webhook {
    fn from(webhook_entity: &WebhookEntity) -> Self {
        Self::new(
            webhook_entity.id.parse().unwrap(),
            webhook_entity.url.parse().unwrap(),
            webhook_entity.request_body.as_str(),
            webhook_entity.description.as_str(),
            webhook_entity.created_at
        )
    }
}


#[derive(Clone, Debug, sqlx::FromRow)]
pub struct JobDoneWatcherEntity {
    pub id: String,
    pub job_name: String,
    pub timeout_seconds: i64,
    pub status: JobDoneWatcherStatusEntity,
    pub created_at: chrono::DateTime<Utc>,
    pub job_done_trigger_webhooks: JobDoneTriggerWebhooksEntity,
}

#[derive(Clone, Debug, sqlx::FromRow, Deserialize)]
pub struct JobDoneTriggerWebhookEntity {
    pub id: String,
    pub webhook_id: String,
    pub timeout_seconds: i64,
    pub status: JobDoneTriggerWebhookStatusEntity,
    pub called_at: Option<chrono::DateTime<Utc>>,
}

impl From<&JobDoneTriggerWebhookEntity> for JobDoneTriggerWebhook {
    fn from(job_done_trigger_webhook_entity: &JobDoneTriggerWebhookEntity) -> Self {
        Self::new(
            Uuid::parse_str(&job_done_trigger_webhook_entity.id).expect("Uuid from db should be correct!"),
            Uuid::parse_str(&job_done_trigger_webhook_entity.webhook_id).expect("Uuid from db should be correct!"),
            job_done_trigger_webhook_entity.timeout_seconds as u32,
            (&job_done_trigger_webhook_entity.status).into(),
            job_done_trigger_webhook_entity.called_at,
        )
    }
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

impl From<JobDoneWatcherEntity> for JobDoneWatcher {
    fn from(job_done_watcher_entity: JobDoneWatcherEntity) -> Self {
        Self::new(
            Uuid::parse_str(&job_done_watcher_entity.id).expect("Uuid from db should be correct!"),
            JobName::new(&job_done_watcher_entity.job_name).expect("Job name should be valid"),
            job_done_watcher_entity.timeout_seconds as u32,
            job_done_watcher_entity.job_done_trigger_webhooks.iter().map(JobDoneTriggerWebhook::from).collect(),
            job_done_watcher_entity.status.into(),
            job_done_watcher_entity.created_at,
        )
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
pub struct JobDoneTriggerWebhooksEntity(Vec<JobDoneTriggerWebhookEntity>);

impl Deref for JobDoneTriggerWebhooksEntity {
    type Target = Vec<JobDoneTriggerWebhookEntity>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for JobDoneTriggerWebhooksEntity {
    fn from(value: String) -> Self {
        if value.contains("\"id\":null") {
            return JobDoneTriggerWebhooksEntity(vec![])
        }

        let result: Vec<JobDoneTriggerWebhookEntity> =
            serde_json::from_str(&value).unwrap_or_else(|err| {
                log::error!("Failed to parse JSON: {:?}", err);
                vec![]
            });
        JobDoneTriggerWebhooksEntity(result)
    }
}


#[derive(Clone, Debug, Copy, Deserialize)]
pub enum JobDoneTriggerWebhookStatusEntity {
    Called,
    NotCalled,
    Failed,
    Timeout,
    Cancelled,
}

#[derive(Clone, Debug, Copy)]
#[derive(PartialEq)]
pub enum JobDoneWatcherStatusEntity {
    Completed,
    PartiallyCompleted,
    Pending,
    Processing,
    Cancelled,
    Failed,
    Timeout,
}

impl Display for JobDoneWatcherStatusEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            JobDoneWatcherStatusEntity::Completed => "Completed".to_string(),
            JobDoneWatcherStatusEntity::PartiallyCompleted => "PartiallyCompleted".to_string(),
            JobDoneWatcherStatusEntity::Pending => "Pending".to_string(),
            JobDoneWatcherStatusEntity::Cancelled => "Cancelled".to_string(),
            JobDoneWatcherStatusEntity::Failed => "Failed".to_string(),
            JobDoneWatcherStatusEntity::Timeout => "Timeout".to_string(),
            JobDoneWatcherStatusEntity::Processing => "Processing".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl Display for JobDoneTriggerWebhookStatusEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            JobDoneTriggerWebhookStatusEntity::Called => "Called",
            JobDoneTriggerWebhookStatusEntity::NotCalled => "NotCalled",
            JobDoneTriggerWebhookStatusEntity::Failed => "Failed",
            JobDoneTriggerWebhookStatusEntity::Timeout => "Timeout",
            JobDoneTriggerWebhookStatusEntity::Cancelled => "Cancelled",
        };
        write!(f, "{}", str)
    }
}

impl From<String> for JobDoneWatcherStatusEntity {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Completed" => JobDoneWatcherStatusEntity::Completed,
            "PartiallyCompleted" => JobDoneWatcherStatusEntity::PartiallyCompleted,
            "Pending" => JobDoneWatcherStatusEntity::Pending,
            "Cancelled" => JobDoneWatcherStatusEntity::Cancelled,
            "Failed" => JobDoneWatcherStatusEntity::Failed,
            "Timeout" => JobDoneWatcherStatusEntity::Timeout,
            "Processing" => JobDoneWatcherStatusEntity::Processing,
            _ => panic!("From<String> JobDoneWatcherStatusEntity"),
        }
    }
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct JobFamilyWatcherEntity {
    pub id: String,
    pub job_family: String,
    pub url: String,
    pub request_body: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

impl From<JobFamilyWatcherEntity> for JobFamilyWatcher {
    fn from(job_family_watcher_entity: JobFamilyWatcherEntity) -> Self {
        Self::new(
            &job_family_watcher_entity.job_family,
            &job_family_watcher_entity.url,
            &job_family_watcher_entity.request_body,
            &job_family_watcher_entity.description,
        ).expect("JobFamilyWatcher::new should not fail for valid JobFamilyWatcherEntity")
    }
}