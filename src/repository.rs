use std::future::Future;

use sqlx::{Database, Pool, Sqlite, SqlitePool};
use sqlx::pool::PoolConnection;

pub use job_done_watchers::get_job_done_watcher_repository;
pub use job_done_watchers::InMemoryJobDoneWatcherRepository;
pub use job_done_watchers::set_job_done_watcher_repository;
pub use webhooks::get_webhook_repository;
pub use webhooks::InMemoryWebhookRepository;
pub use webhooks::set_webhook_repository;

mod webhooks;
mod job_done_watchers;

#[async_trait::async_trait]
pub trait AsyncLockGuard<T> {
    async fn lock(&self, id: &str, critical_section: Box<dyn FnOnce(T) -> Box<dyn Future<Output=()> + Send> + Send>);
}


#[async_trait::async_trait]
pub trait SqlxAcquire {
    type DB: Database;
    async fn acquire(&self) ->  anyhow::Result<PoolConnection<Self::DB>>;
}

pub struct SqliteDatabase {
    pool_connection: Pool<Sqlite>,
}

impl SqliteDatabase {
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        Ok(Self {
            pool_connection: SqlitePool::connect(url).await?
        })
    }
}

#[async_trait::async_trait]
impl SqlxAcquire for SqliteDatabase {
    type DB = Sqlite;

    async fn acquire(&self) -> anyhow::Result<PoolConnection<Self::DB>> {
        Ok(self.acquire().await?)
    }
}