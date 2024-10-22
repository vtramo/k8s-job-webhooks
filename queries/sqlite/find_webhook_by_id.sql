SELECT id, url, request_body, description, created_at AS "created_at: _"
FROM webhooks
WHERE id = ?1