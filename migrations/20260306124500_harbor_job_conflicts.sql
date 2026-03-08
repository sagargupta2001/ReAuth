CREATE TABLE IF NOT EXISTS harbor_job_conflicts (
    id TEXT PRIMARY KEY,
    job_id TEXT NOT NULL,
    resource_key TEXT NOT NULL,
    action TEXT NOT NULL,
    policy TEXT NOT NULL,
    original_id TEXT,
    resolved_id TEXT,
    message TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_harbor_job_conflicts_job
    ON harbor_job_conflicts (job_id, created_at);
