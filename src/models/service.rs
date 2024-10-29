use std::fmt;

use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;
use yaml_rust2::Yaml;

pub use http_url::HttpUrl;
pub use job_name::{JobName, JobNameError};


mod job_name;

mod http_url;

#[derive(Debug, Clone)]
pub struct CreateWebhookRequest {
    url: HttpUrl,
    request_body: String,
    description: String,
}

#[derive(Debug, Error)]
pub enum CreateWebhookRequestError {
    #[error("Invalid URL format")]
    InvalidHttpUrl(#[from] http_url::HttpUrlError),
}

impl CreateWebhookRequest {
    pub fn new(url: &str, request_body: &str, description: &str) -> Result<Self, CreateWebhookRequestError> {
        Ok(Self {
            url: HttpUrl::new(url)?,
            request_body: request_body.to_string(),
            description: description.to_string(),
        })
    }

    pub fn url(&self) -> &HttpUrl {
        &self.url
    }
    pub fn request_body(&self) -> &str {
        &self.request_body
    }
    pub fn description(&self) -> &str {
        &self.description
    }
}


#[derive(Clone, Debug)]
pub struct Webhook {
    id: Uuid,
    url: HttpUrl,
    request_body: String,
    description: String,
    created_at: DateTime<Utc>,
}

impl Webhook {
    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn url(&self) -> &HttpUrl {
        &self.url
    }
    pub fn request_body(&self) -> &str {
        &self.request_body
    }
    pub fn description(&self) -> &str {
        &self.description
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn new(id: Uuid, url: HttpUrl, request_body: &str, description: &str, created_at: DateTime<Utc>) -> Self {
        Self {
            id,
            url,
            request_body: request_body.to_string(),
            description: description.to_string(),
            created_at
        }
    }
}


#[derive(Clone, Debug)]
pub struct CreateJobDoneWatcherRequest {
    job_name: JobName,
    timeout_seconds: u32,
    job_done_trigger_webhooks: Vec<CreateJobDoneTriggerWebhookRequest>,
}

impl CreateJobDoneWatcherRequest {
    pub fn new(
        job_name: &str,
        timeout_seconds: u32,
        job_done_trigger_webhooks: Vec<CreateJobDoneTriggerWebhookRequest>
    ) -> anyhow::Result<Self> {
        let job_name = JobName::new(job_name)?;
        Ok(Self { job_name, timeout_seconds, job_done_trigger_webhooks })
    }

    pub fn job_name(&self) -> &JobName {
        &self.job_name
    }
    pub fn timeout_seconds(&self) -> u32 {
        self.timeout_seconds
    }
    pub fn job_done_trigger_webhooks(&self) -> &Vec<CreateJobDoneTriggerWebhookRequest> {
        &self.job_done_trigger_webhooks
    }
}

#[derive(Clone, Debug)]
pub struct CreateJobDoneTriggerWebhookRequest {
    webhook_id: Uuid,
    timeout_seconds: u32,
}

#[derive(Debug, Error)]
pub enum CreateJobDoneTriggerWebhookRequestError {
    #[error("Invalid webhook identifier")]
    InvalidWebhookId(#[from] uuid::Error)
}

impl CreateJobDoneTriggerWebhookRequest {
    pub fn new(webhook_id: &str, timeout_seconds: u32) -> Result<Self, CreateJobDoneTriggerWebhookRequestError> {
        let webhook_uuid = Uuid::parse_str(webhook_id)?;
        Ok(Self { webhook_id: webhook_uuid, timeout_seconds })
    }

    pub fn webhook_id(&self) -> Uuid {
        self.webhook_id
    }
    pub fn timeout_seconds(&self) -> u32 {
        self.timeout_seconds
    }
}


#[derive(Clone, Debug)]
pub struct JobDoneTriggerWebhook {
    id: Uuid,
    webhook_id: Uuid,
    timeout_seconds: u32,
    status: JobDoneTriggerWebhookStatus,
    called_at: Option<DateTime<Utc>>,
}

impl JobDoneTriggerWebhook {
    pub fn new(id: Uuid, webhook_id: Uuid, timeout_seconds: u32, status: JobDoneTriggerWebhookStatus, called_at: Option<DateTime<Utc>>) -> Self {
        Self { id, webhook_id, timeout_seconds, status, called_at }
    }

    pub fn set_called_at(&mut self, date_time: DateTime<Utc>) {
        self.called_at = Some(date_time);
    }

    pub fn set_status(&mut self, status: JobDoneTriggerWebhookStatus) {
        self.status = status;
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn webhook_id(&self) -> Uuid {
        self.webhook_id
    }

    pub fn timeout_seconds(&self) -> u32 {
        self.timeout_seconds
    }

    pub fn status(&self) -> &JobDoneTriggerWebhookStatus {
        &self.status
    }
    pub fn called_at(&self) -> Option<DateTime<Utc>> {
        self.called_at
    }
}

#[derive(Clone, Debug, Copy)]
pub enum JobDoneTriggerWebhookStatus {
    Called,
    NotCalled,
    Failed,
    Timeout,
    Cancelled,
}

impl fmt::Display for JobDoneTriggerWebhookStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_str = match self {
            JobDoneTriggerWebhookStatus::Called => "Called",
            JobDoneTriggerWebhookStatus::NotCalled => "Not Called",
            JobDoneTriggerWebhookStatus::Failed => "Failed",
            JobDoneTriggerWebhookStatus::Timeout => "Timeout",
            JobDoneTriggerWebhookStatus::Cancelled => "Cancelled",
        };
        write!(f, "{}", status_str)
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
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


#[derive(Clone, Debug)]
pub struct JobDoneWatcher {
    id: Uuid,
    job_name: JobName,
    timeout_seconds: u32,
    status: JobDoneWatcherStatus,
    created_at: DateTime<Utc>,
    job_done_trigger_webhooks: Vec<JobDoneTriggerWebhook>,
}

impl JobDoneWatcher {
    pub fn new(
        id: Uuid,
        job_name: JobName,
        timeout_seconds: u32,
        job_done_trigger_webhooks: Vec<JobDoneTriggerWebhook>,
        status: JobDoneWatcherStatus,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self { id, job_name, timeout_seconds, status, created_at, job_done_trigger_webhooks }
    }

    pub fn set_status(&mut self, status: JobDoneWatcherStatus) {
        self.status = status;
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn job_name(&self) -> &str {
        &self.job_name
    }

    pub fn timeout_seconds(&self) -> u32 {
        self.timeout_seconds
    }

    pub fn status(&self) -> JobDoneWatcherStatus {
        self.status.clone()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn job_done_trigger_webhooks(&self) -> &Vec<JobDoneTriggerWebhook> {
        &self.job_done_trigger_webhooks
    }

    pub fn job_done_trigger_webhooks_mut(&mut self) -> &mut Vec<JobDoneTriggerWebhook> {
        &mut self.job_done_trigger_webhooks
    }
}


#[derive(Clone, Debug)]
pub struct JobFamilyWatcher {
    job_family: String,
    url: HttpUrl,
    request_body: String,
    description: String
}

impl JobFamilyWatcher {
    pub fn new(job_family: &str, url: &str, request_body: &str, description: &str) -> anyhow::Result<Self> {
        Ok(Self {
            job_family: job_family.to_string(),
            url: HttpUrl::new(url)?,
            request_body: request_body.to_string(),
            description: description.to_string()
        })
    }

    pub fn job_family(&self) -> &str {
        &self.job_family
    }

    pub fn url(&self) -> &HttpUrl {
        &self.url
    }

    pub fn request_body(&self) -> &str {
        &self.request_body
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

impl TryFrom<Yaml> for JobFamilyWatcher {
    type Error = anyhow::Error;

    fn try_from(yaml: Yaml) -> Result<Self, Self::Error> {
        let job_family = extract_yaml_string(&yaml, "jobFamily")?;
        let url = extract_yaml_string(&yaml, "url")?;
        let request_body = extract_yaml_string(&yaml, "requestBody").unwrap_or_default();
        let description = extract_yaml_string(&yaml, "description").unwrap_or_default();

        Ok(Self::new(
            &job_family,
            &url,
            &request_body,
            &description,
        )?)
    }
}


fn extract_yaml_string(yaml: &Yaml, key: &str) -> Result<String, anyhow::Error> {
    match &yaml[key] {
        Yaml::String(value) => Ok(value.clone()),
        _ => Err(anyhow::anyhow!("Missing or invalid value for key: {}", key)),
    }
}