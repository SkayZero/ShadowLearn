-- Migration: Create conversations table
CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    app_context TEXT
);

-- Migration: Create messages table
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    metadata TEXT,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);

-- Migration: Create contexts table
CREATE TABLE IF NOT EXISTS contexts (
    id TEXT PRIMARY KEY,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    app_name TEXT NOT NULL,
    app_bundle_id TEXT NOT NULL,
    window_title TEXT NOT NULL,
    clipboard_content TEXT,
    idle_seconds REAL NOT NULL,
    screenshot_data TEXT,
    capture_duration_ms INTEGER NOT NULL DEFAULT 0
);

-- Migration: Create preferences table
CREATE TABLE IF NOT EXISTS preferences (
    id TEXT PRIMARY KEY,
    key TEXT UNIQUE NOT NULL,
    value TEXT NOT NULL,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for better performance
CREATE INDEX IF NOT EXISTS idx_conversations_updated_at ON conversations(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_messages_conversation_id ON messages(conversation_id);
CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_contexts_app_name ON contexts(app_name);
CREATE INDEX IF NOT EXISTS idx_contexts_timestamp ON contexts(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_preferences_key ON preferences(key);
