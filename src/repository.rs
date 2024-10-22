use sqlx::{Database, Pool, Sqlite, SqlitePool};
use sqlx::pool::PoolConnection;
use sqlx::sqlite::SqlitePoolOptions;

pub use job_done_watchers::get_job_done_watcher_repository;
pub use job_done_watchers::InMemoryJobDoneWatcherRepository;
pub use job_done_watchers::set_job_done_watcher_repository;
pub use job_family_watcher::get_job_family_watcher_repository;
pub use job_family_watcher::set_job_family_watcher_repository;
pub use webhooks::get_webhook_repository;
pub use webhooks::InMemoryWebhookRepository;
pub use webhooks::set_webhook_repository;

mod webhooks;
mod job_done_watchers;
mod job_family_watcher;

#[derive(Clone)]
pub struct SqliteDatabase {
    pool_connection: Pool<Sqlite>,
}

impl SqliteDatabase {
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        Ok(Self {
            pool_connection: SqlitePool::connect(url).await?
        })
    }

    pub async fn connect_in_memory(url: &str) -> anyhow::Result<Self> {
        Ok(Self {
            pool_connection: SqlitePoolOptions::new()
                .min_connections(1)
                .max_connections(1)
                .idle_timeout(None)
                .max_lifetime(None)
                .connect(url).await?
        })
    }
}

#[async_trait::async_trait]
pub trait SqlxAcquire {
    type DB: Database;
    async fn acquire(&self) ->  anyhow::Result<PoolConnection<Self::DB>>;
}

#[async_trait::async_trait]
impl SqlxAcquire for SqliteDatabase {
    type DB = Sqlite;

    async fn acquire(&self) -> anyhow::Result<PoolConnection<Self::DB>> {
        Ok(self.pool_connection.acquire().await?)
    }
}