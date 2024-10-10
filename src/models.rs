use k8s_openapi::serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all="camelCase")]
pub struct CronJobsMonitor {
    cronjobs: Vec<String>,
    webhooks: Vec<WebHook>
}

impl CronJobsMonitor {
    pub fn get_cronjobs(&self) -> Vec<String> {
        self.cronjobs.clone()
    }

    pub fn get_webhooks(&self) -> Vec<WebHook> {
        self.webhooks.clone()
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all="camelCase")]
pub struct JobsMonitor {
    jobs: Vec<String>,
    webhooks: Vec<WebHook>
}

impl JobsMonitor {
    pub fn get_jobs(&self) -> Vec<String> {
        self.jobs.clone()
    }

    pub fn get_webhooks(&self) -> Vec<WebHook> {
        self.webhooks.clone()
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all="camelCase")]
pub struct WebHook {
    url: String,
    request_body: String,
}

impl WebHook {
    pub fn get_url(&self) -> String {
        self.url.clone()
    }

    pub fn get_request_body(&self) -> String {
        self.request_body.clone()
    }
}