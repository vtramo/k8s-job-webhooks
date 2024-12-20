SELECT
    job_done_watchers.id,
    job_done_watchers.job_name,
    job_done_watchers.timeout_seconds,
    job_done_watchers.status,
    job_done_watchers.created_at AS "created_at: _",
    coalesce(json_group_array(json_object(
        'id', job_done_trigger_webhooks.id,
        'webhook_id', job_done_trigger_webhooks.webhook_id,
        'timeout_seconds', job_done_trigger_webhooks.timeout_seconds,
        'status', job_done_trigger_webhooks.status,
        'called_at', job_done_trigger_webhooks.called_at)), json_object()) AS "job_done_trigger_webhooks!: String"
FROM
    job_done_watchers
LEFT JOIN
    job_done_trigger_webhooks ON job_done_watchers.id = job_done_trigger_webhooks.job_done_watcher_id
WHERE
    job_done_watchers.job_name = ?1 AND job_done_watchers.status = ?2
GROUP BY
    job_done_watchers.id