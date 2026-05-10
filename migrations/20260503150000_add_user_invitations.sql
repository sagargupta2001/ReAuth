CREATE TABLE invitations (
    id TEXT PRIMARY KEY NOT NULL,
    realm_id TEXT NOT NULL,
    email TEXT NOT NULL,
    email_normalized TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'accepted', 'expired', 'revoked')),
    token_hash TEXT NOT NULL,
    expiry_days INTEGER NOT NULL,
    expires_at DATETIME NOT NULL,
    invited_by_user_id TEXT,
    accepted_user_id TEXT,
    accepted_at DATETIME,
    revoked_at DATETIME,
    resend_count INTEGER NOT NULL DEFAULT 0,
    last_sent_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (realm_id) REFERENCES realms (id) ON DELETE CASCADE,
    FOREIGN KEY (invited_by_user_id) REFERENCES users (id) ON DELETE SET NULL,
    FOREIGN KEY (accepted_user_id) REFERENCES users (id) ON DELETE SET NULL
);

CREATE INDEX idx_invitations_realm_created_at
    ON invitations (realm_id, created_at DESC);
CREATE INDEX idx_invitations_realm_status_created_at
    ON invitations (realm_id, status, created_at DESC);
CREATE INDEX idx_invitations_realm_email_normalized
    ON invitations (realm_id, email_normalized);
CREATE INDEX idx_invitations_realm_expires_at
    ON invitations (realm_id, expires_at);
CREATE UNIQUE INDEX idx_invitations_token_hash
    ON invitations (token_hash);
CREATE UNIQUE INDEX idx_invitations_pending_email_unique
    ON invitations (realm_id, email_normalized)
    WHERE status = 'pending';
