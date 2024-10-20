use std::fmt::{Display, Formatter};
use std::ops::Deref;

use k8s_openapi::serde_json;
use serde::Deserialize;

#[derive(sqlx::FromRow, Debug)]
pub struct WebhookEntity {
    pub id: String,
    pub url: String,
    pub request_body: String,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct JobDoneWatcherEntity {
    pub id: String,
    pub job_name: String,
    pub timeout_seconds: i64,
    pub status: JobDoneWatcherStatusEntity,
    pub created_at: chrono::NaiveDateTime,
    pub job_done_trigger_webhooks: JobDoneTriggerWebhooksEntity,
}

#[derive(Clone, Debug, sqlx::FromRow, Deserialize)]
pub struct JobDoneTriggerWebhookEntity {
    pub id: String,
    pub webhook_id: String,
    pub timeout_seconds: i64,
    pub status: JobDoneTriggerWebhookStatusEntity,
    pub called_at: Option<chrono::NaiveDateTime>,
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

        let result: Vec<JobDoneTriggerWebhookEntity> = serde_json::from_str(&value).unwrap_or(vec![]);
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
            _ => panic!()
        }
    }
}