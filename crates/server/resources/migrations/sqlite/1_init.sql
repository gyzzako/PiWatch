CREATE TABLE IF NOT EXISTS agents (
    agent_id TEXT PRIMARY KEY NOT NULL,
    secret_hash TEXT NOT NULL,
    salt TEXT NOT NULL,
    hostname TEXT NOT NULL,
    agent_version TEXT NOT NULL,
    ipv4 TEXT,
    last_seen INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    revoked INTEGER NOT NULL DEFAULT 0,
    deactivated_at INTEGER
);
CREATE TABLE IF NOT EXISTS install_tokens (
    token_hash TEXT PRIMARY KEY NOT NULL,
    salt TEXT NOT NULL DEFAULT ''
);
