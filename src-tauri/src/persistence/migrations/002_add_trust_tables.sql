-- Migration 002: Add J19 Trust and Learning tables (simplified)

-- User trust table (J19)
CREATE TABLE IF NOT EXISTS user_trust (
    id TEXT PRIMARY KEY,
    device_id TEXT UNIQUE NOT NULL,
    pos REAL NOT NULL DEFAULT 0.0,
    neg REAL NOT NULL DEFAULT 0.0,
    trust REAL NOT NULL DEFAULT 0.5,
    quarantine BOOLEAN NOT NULL DEFAULT 0,
    last_updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Trust events table (J19)
CREATE TABLE IF NOT EXISTS trust_events (
    id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL,
    reward REAL NOT NULL,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Outcomes table (J19)
CREATE TABLE IF NOT EXISTS outcomes (
    id TEXT PRIMARY KEY,
    suggestion_id TEXT NOT NULL,
    used BOOLEAN NOT NULL DEFAULT 0,
    helpful BOOLEAN NOT NULL DEFAULT 0,
    reverted BOOLEAN NOT NULL DEFAULT 0,
    time_to_flow_ms INTEGER NOT NULL DEFAULT 0,
    reward REAL NOT NULL,
    cluster_id TEXT NOT NULL,
    artefact_type TEXT NOT NULL,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
