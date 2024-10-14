use std::collections::HashMap;
use std::sync::Mutex;

pub mod webhooks;
pub mod job_done_watchers;

pub static IDEMPOTENCY_KEY_HEADER: &'static str = "Idempotency-Key";

#[derive(Debug)]
pub struct IdempotencyMap {
    resource_id_by_idempotency_id: Mutex<HashMap<String, String>>,
}

impl IdempotencyMap {
    pub fn new() -> Self {
        Self {
            resource_id_by_idempotency_id: Mutex::new(HashMap::with_capacity(5)),
        }
    }

    pub fn get_resource_id(&self, idempotency_id: &str) -> Option<String> {
        self.resource_id_by_idempotency_id
            .lock()
            .unwrap()
            .get(idempotency_id)
            .cloned()
    }

    pub fn insert(&self, idempotency_id: &str, resource_id: &str) {
        let mut resource_id_by_idempotency_id = self.resource_id_by_idempotency_id
            .lock()
            .unwrap();

        resource_id_by_idempotency_id.insert(idempotency_id.to_string(), resource_id.to_string());
    }
}