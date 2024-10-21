-- Add migration script here
CREATE TABLE IF NOT EXISTS webhooks
(
    id VARCHAR PRIMARY KEY NOT NULL,
    url VARCHAR NOT NULL,
    request_body VARCHAR NOT NULL,
    description TEXT NOT NULL,
    created_at DATETIME NOT NULL
);

CREATE TABLE IF NOT EXISTS job_done_trigger_webhooks
(
    id VARCHAR PRIMARY KEY NOT NULL,
    webhook_id VARCHAR NOT NULL,
    job_done_watcher_id VARCHAR NOT NULL,
    timeout_seconds INTEGER NOT NULL DEFAULT 0,
    status VARCHAR NOT NULL,
    called_at DATETIME DEFAULT NULL,
    FOREIGN KEY(webhook_id) REFERENCES webhooks(id),
    FOREIGN KEY(job_done_watcher_id) REFERENCES job_done_watchers(id)
);

CREATE TABLE IF NOT EXISTS job_done_watchers
(
    id VARCHAR PRIMARY KEY NOT NULL,
    job_name VARCHAR NOT NULL,
    timeout_seconds INTEGER NOT NULL DEFAULT 0,
    status VARCHAR NOT NULL,
    created_at DATETIME NOT NULL
);