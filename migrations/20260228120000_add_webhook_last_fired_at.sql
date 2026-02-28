-- Add last_fired_at to webhook_endpoints
ALTER TABLE webhook_endpoints ADD COLUMN last_fired_at DATETIME;
