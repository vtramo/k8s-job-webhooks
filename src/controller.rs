use moka::sync::Cache;


pub mod webhooks;
pub mod job_done_watchers;

pub static IDEMPOTENCY_KEY_HEADER: &'static str = "Idempotency-Key";

#[derive(Debug)]
pub struct IdempotencyMap {
    resource_id_by_idempotency_id: Cache<String, String>,
}

impl IdempotencyMap {
    pub fn new() -> Self {
        Self {
            resource_id_by_idempotency_id: Cache::new(25),
        }
    }

    pub fn get_resource_id(&self, idempotency_id: &str) -> Option<String> {
        self.resource_id_by_idempotency_id.get(idempotency_id)
    }

    pub fn insert(&self, idempotency_id: &str, resource_id: &str) {
        self.resource_id_by_idempotency_id.insert(idempotency_id.to_string(), resource_id.to_string());
    }
}