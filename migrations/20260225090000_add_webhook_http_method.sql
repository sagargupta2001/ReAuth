ALTER TABLE webhook_endpoints ADD COLUMN http_method TEXT NOT NULL DEFAULT 'POST';

UPDATE webhook_endpoints
SET http_method = 'POST'
WHERE http_method IS NULL;
