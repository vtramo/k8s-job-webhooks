use moka::sync::Cache;
use uuid::Uuid;


pub mod webhooks;
pub mod job_done_watchers;

pub static IDEMPOTENCY_KEY_HEADER: &'static str = "Idempotency-Key";

#[derive(Debug)]
pub struct IdempotencyMap {
    resource_id_by_idempotency_id: Cache<Uuid, Uuid>,
}

impl IdempotencyMap {
    pub fn new() -> Self {
        Self {
            resource_id_by_idempotency_id: Cache::new(25),
        }
    }

    pub fn get_resource_id(&self, idempotency_id: &Uuid) -> Option<Uuid> {
        self.resource_id_by_idempotency_id.get(idempotency_id)
    }

    pub fn insert(&self, idempotency_id: &Uuid, resource_id: &Uuid) {
        self.resource_id_by_idempotency_id.insert(idempotency_id.clone(), resource_id.clone());
    }
}