SELECT id, job_family, url, request_body, description, created_at AS "created_at: _"
FROM job_watcher_family
WHERE job_family = ?1
