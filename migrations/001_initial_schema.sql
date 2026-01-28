-- Initial schema for Ragnoria RO2 Server
-- SQLite version (MySQL version will be in separate migration)

-- Accounts table
CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT UNIQUE NOT NULL COLLATE NOCASE,
    password_hash TEXT NOT NULL,
    email TEXT,
    created_at INTEGER NOT NULL,  -- Unix timestamp
    last_login INTEGER,            -- Unix timestamp
    is_banned INTEGER DEFAULT 0,   -- Boolean (0 = false, 1 = true)
    ban_reason TEXT,
    ban_until INTEGER,             -- Unix timestamp, NULL = permanent
    is_gm INTEGER DEFAULT 0,       -- Boolean (0 = false, 1 = true)
    gm_level INTEGER DEFAULT 0     -- 0 = player, 1-99 = GM levels
);

-- Sessions table (for active login sessions)
CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL,
    session_key TEXT UNIQUE NOT NULL,  -- 32-byte hex string from AnsLogin
    created_at INTEGER NOT NULL,        -- Unix timestamp
    expires_at INTEGER NOT NULL,        -- Unix timestamp
    ip_address TEXT NOT NULL,
    last_activity INTEGER NOT NULL,     -- Unix timestamp for timeout detection
    server_id INTEGER,                  -- Which world server (NULL = lobby only)
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

-- Characters table
CREATE TABLE IF NOT EXISTS characters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL,
    name TEXT UNIQUE NOT NULL COLLATE NOCASE,
    class_id INTEGER NOT NULL,          -- Class/job ID
    level INTEGER DEFAULT 1,
    experience BIGINT DEFAULT 0,
    map_id INTEGER NOT NULL,            -- Current map
    position_x REAL NOT NULL,           -- X coordinate
    position_y REAL NOT NULL,           -- Y coordinate  
    position_z REAL NOT NULL,           -- Z coordinate
    hp INTEGER NOT NULL,
    max_hp INTEGER NOT NULL,
    mp INTEGER NOT NULL,
    max_mp INTEGER NOT NULL,
    gold BIGINT DEFAULT 0,
    created_at INTEGER NOT NULL,        -- Unix timestamp
    last_played INTEGER,                -- Unix timestamp
    deleted_at INTEGER,                 -- Unix timestamp (soft delete), NULL = active
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

-- Character stats table (STR, DEX, INT, etc.)
CREATE TABLE IF NOT EXISTS character_stats (
    character_id INTEGER PRIMARY KEY,
    strength INTEGER DEFAULT 1,
    dexterity INTEGER DEFAULT 1,
    intelligence INTEGER DEFAULT 1,
    vitality INTEGER DEFAULT 1,
    luck INTEGER DEFAULT 1,
    stat_points INTEGER DEFAULT 0,      -- Unallocated points
    FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
);

-- Inventory table
CREATE TABLE IF NOT EXISTS inventory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    character_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,           -- Item template ID
    quantity INTEGER DEFAULT 1,
    slot_index INTEGER,                 -- Inventory slot position
    is_equipped INTEGER DEFAULT 0,      -- Boolean
    enchantment_level INTEGER DEFAULT 0,
    FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_accounts_username ON accounts(username);
CREATE INDEX IF NOT EXISTS idx_sessions_account_id ON sessions(account_id);
CREATE INDEX IF NOT EXISTS idx_sessions_session_key ON sessions(session_key);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_characters_account_id ON characters(account_id);
CREATE INDEX IF NOT EXISTS idx_characters_name ON characters(name);
CREATE INDEX IF NOT EXISTS idx_inventory_character_id ON inventory(character_id);

-- Insert default admin account for testing
-- Password: "admin123" (bcrypt hash)
INSERT OR IGNORE INTO accounts (username, password_hash, created_at, is_gm, gm_level)
VALUES ('admin', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5ufFb0xVRz3Ei', strftime('%s', 'now'), 1, 99);

-- Insert test player account
-- Password: "player123" (bcrypt hash)  
INSERT OR IGNORE INTO accounts (username, password_hash, created_at, is_gm, gm_level)
VALUES ('player', '$2b$12$K3QGQjxkm3F9KfEXUXO.weZWZpP5FfJzZOJWQh.xYqWJsqZqOqF5O', strftime('%s', 'now'), 0, 0);
