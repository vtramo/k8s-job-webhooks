use async_trait::async_trait;
pub use webhooks::get_webhook_repository;
pub use webhooks::set_webhook_repository;
pub use webhooks::InMemoryWebhookRepository;

pub use job_done_watchers::get_job_done_watcher_repository;
pub use job_done_watchers::set_job_done_watcher_repository;
pub use job_done_watchers::InMemoryJobDoneWatcherRepository;

mod webhooks;
mod job_done_watchers;

#[async_trait]
pub trait CrudRepository: Send + Sync {
    type Entity;

    async fn find_all(&self) -> Vec<Self::Entity>;
    async fn find_by_id(&self, id: &str) -> Option<Self::Entity>;
    async fn save(&self, entity: Self::Entity);
}