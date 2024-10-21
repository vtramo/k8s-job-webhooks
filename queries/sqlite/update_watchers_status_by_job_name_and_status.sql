UPDATE job_done_watchers
SET status = ?3
WHERE job_done_watchers.job_name = ?1 AND job_done_watchers.status = ?2
RETURNING job_done_watchers.id