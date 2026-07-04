-- AGENT
CREATE TABLE IF NOT EXISTS agent (
    agent_id TEXT PRIMARY KEY NOT NULL,
    hostname TEXT NOT NULL,
    agent_version TEXT NOT NULL,
    ipv4 TEXT,
    last_seen INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    revoked INTEGER NOT NULL DEFAULT 0,
    deactivated_at INTEGER
);
CREATE INDEX IF NOT EXISTS idx_agent_hostname ON agent(hostname);


-- AGENT AUTH
CREATE TABLE IF NOT EXISTS agent_auth (
    agent_auth_id TEXT PRIMARY KEY NOT NULL,
    fk_agent_id TEXT NOT NULL,
    secret_hash TEXT NOT NULL,
    salt TEXT NOT NULL,
    FOREIGN KEY (fk_agent_id) REFERENCES agent(agent_id) ON DELETE CASCADE
);

-- INSTALL TOKEN
CREATE TABLE IF NOT EXISTS install_token (
    token_hash TEXT PRIMARY KEY NOT NULL,
    salt TEXT NOT NULL,
    expires_at INTEGER,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_install_token_expires ON install_token(expires_at);
