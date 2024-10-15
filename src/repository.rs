pub use webhooks::get_webhook_repository;
pub use webhooks::set_webhook_repository;
pub use webhooks::InMemoryWebhookRepository;

pub use job_done_watchers::get_job_done_watcher_repository;
pub use job_done_watchers::set_job_done_watcher_repository;
pub use job_done_watchers::InMemoryJobDoneWatcherRepository;

mod webhooks;
mod job_done_watchers;

pub trait CrudRepository: Send + Sync {
    type Entity;

    fn find_all(&self) -> Vec<Self::Entity>;
    fn find_by_id(&self, id: &str) -> Option<Self::Entity>;
    fn save(&self, entity: Self::Entity);
}