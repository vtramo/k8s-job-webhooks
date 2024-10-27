-- Add migration script here
CREATE INDEX IF NOT EXISTS watchers_job_name_and_status_idx
ON job_done_watchers (job_name, status);

CREATE INDEX IF NOT EXISTS watchers_job_status_idx
ON job_done_watchers (status);

CREATE INDEX IF NOT EXISTS job_watcher_family_idx
ON job_watcher_family (job_family);

