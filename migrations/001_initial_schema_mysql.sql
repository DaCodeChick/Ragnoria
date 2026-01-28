-- MySQL version of initial schema
-- Use this for production deployments

-- Accounts table
CREATE TABLE IF NOT EXISTS accounts (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(32) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    created_at BIGINT UNSIGNED NOT NULL,
    last_login BIGINT UNSIGNED,
    is_banned TINYINT(1) DEFAULT 0,
    ban_reason TEXT,
    ban_until BIGINT UNSIGNED,
    is_gm TINYINT(1) DEFAULT 0,
    gm_level TINYINT UNSIGNED DEFAULT 0,
    INDEX idx_username (username)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    account_id INT UNSIGNED NOT NULL,
    session_key VARCHAR(64) UNIQUE NOT NULL,
    created_at BIGINT UNSIGNED NOT NULL,
    expires_at BIGINT UNSIGNED NOT NULL,
    ip_address VARCHAR(45) NOT NULL,
    last_activity BIGINT UNSIGNED NOT NULL,
    server_id INT UNSIGNED,
    INDEX idx_account_id (account_id),
    INDEX idx_session_key (session_key),
    INDEX idx_expires_at (expires_at),
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Characters table
CREATE TABLE IF NOT EXISTS characters (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    account_id INT UNSIGNED NOT NULL,
    name VARCHAR(32) UNIQUE NOT NULL,
    class_id INT UNSIGNED NOT NULL,
    level INT UNSIGNED DEFAULT 1,
    experience BIGINT UNSIGNED DEFAULT 0,
    map_id INT UNSIGNED NOT NULL,
    position_x FLOAT NOT NULL,
    position_y FLOAT NOT NULL,
    position_z FLOAT NOT NULL,
    hp INT UNSIGNED NOT NULL,
    max_hp INT UNSIGNED NOT NULL,
    mp INT UNSIGNED NOT NULL,
    max_mp INT UNSIGNED NOT NULL,
    gold BIGINT UNSIGNED DEFAULT 0,
    created_at BIGINT UNSIGNED NOT NULL,
    last_played BIGINT UNSIGNED,
    deleted_at BIGINT UNSIGNED,
    INDEX idx_account_id (account_id),
    INDEX idx_name (name),
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Character stats table
CREATE TABLE IF NOT EXISTS character_stats (
    character_id INT UNSIGNED PRIMARY KEY,
    strength INT UNSIGNED DEFAULT 1,
    dexterity INT UNSIGNED DEFAULT 1,
    intelligence INT UNSIGNED DEFAULT 1,
    vitality INT UNSIGNED DEFAULT 1,
    luck INT UNSIGNED DEFAULT 1,
    stat_points INT UNSIGNED DEFAULT 0,
    FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Inventory table
CREATE TABLE IF NOT EXISTS inventory (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    character_id INT UNSIGNED NOT NULL,
    item_id INT UNSIGNED NOT NULL,
    quantity INT UNSIGNED DEFAULT 1,
    slot_index INT UNSIGNED,
    is_equipped TINYINT(1) DEFAULT 0,
    enchantment_level TINYINT UNSIGNED DEFAULT 0,
    INDEX idx_character_id (character_id),
    FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Insert default admin account (password: admin123)
INSERT IGNORE INTO accounts (username, password_hash, created_at, is_gm, gm_level)
VALUES ('admin', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5ufFb0xVRz3Ei', UNIX_TIMESTAMP(), 1, 99);

-- Insert test player account (password: player123)
INSERT IGNORE INTO accounts (username, password_hash, created_at, is_gm, gm_level)
VALUES ('player', '$2b$12$K3QGQjxkm3F9KfEXUXO.weZWZpP5FfJzZOJWQh.xYqWJsqZqOqF5O', UNIX_TIMESTAMP(), 0, 0);
