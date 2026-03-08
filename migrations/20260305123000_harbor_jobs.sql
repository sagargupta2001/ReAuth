CREATE TABLE IF NOT EXISTS harbor_jobs (
    id TEXT PRIMARY KEY,
    realm_id TEXT NOT NULL,
    job_type TEXT NOT NULL,
    status TEXT NOT NULL,
    scope TEXT NOT NULL,
    total_resources INTEGER NOT NULL DEFAULT 0,
    processed_resources INTEGER NOT NULL DEFAULT 0,
    created_count INTEGER NOT NULL DEFAULT 0,
    updated_count INTEGER NOT NULL DEFAULT 0,
    dry_run INTEGER NOT NULL DEFAULT 0,
    conflict_policy TEXT,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_harbor_jobs_realm_created_at
    ON harbor_jobs (realm_id, created_at DESC);
