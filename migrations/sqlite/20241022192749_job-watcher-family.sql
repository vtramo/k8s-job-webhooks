CREATE TABLE IF NOT EXISTS job_watcher_family
(
    id VARCHAR PRIMARY KEY NOT NULL,
    job_family VARCHAR NOT NULL,
    url VARCHAR NOT NULL,
    request_body VARCHAR NOT NULL,
    description TEXT NOT NULL,
    created_at DATETIME NOT NULL
);