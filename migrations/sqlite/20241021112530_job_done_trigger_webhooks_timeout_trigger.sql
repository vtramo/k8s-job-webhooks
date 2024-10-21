-- Add migration script here
CREATE TRIGGER IF NOT EXISTS job_done_trigger_webhooks_timeout_trigger
    AFTER UPDATE OF status
    ON job_done_watchers
    WHEN new.status = 'Timeout'
BEGIN
    UPDATE job_done_trigger_webhooks
    SET status = 'Timeout'
    WHERE job_done_trigger_webhooks.job_done_watcher_id = new.id;
END;