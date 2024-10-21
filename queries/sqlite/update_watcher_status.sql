UPDATE job_done_watchers
SET status = 'Timeout'
WHERE job_done_watchers.status = 'Pending' AND job_done_watchers.id = ?1