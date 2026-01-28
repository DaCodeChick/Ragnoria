# Database Migrations

This directory contains SQL migration files for the Ragnoria database schema.

## Files

- **`001_initial_schema.sql`** - SQLite version (for development/testing)
- **`001_initial_schema_mysql.sql`** - MySQL version (for production)

## Running Migrations

### SQLite (Development)

```bash
# Create database and run migrations
sqlite3 ragnoria.db < migrations/001_initial_schema.sql

# Or use sqlx CLI
sqlx database create --database-url sqlite:ragnoria.db
sqlx migrate run --database-url sqlite:ragnoria.db
```

### MySQL (Production)

```bash
# Create database
mysql -u root -p -e "CREATE DATABASE ragnoria CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"

# Run migrations
mysql -u root -p ragnoria < migrations/001_initial_schema_mysql.sql

# Or use sqlx CLI
sqlx database create --database-url mysql://user:pass@localhost/ragnoria
sqlx migrate run --database-url mysql://user:pass@localhost/ragnoria
```

## Schema Overview

### Tables

**accounts**
- Stores user authentication data
- Includes ban status and GM levels
- Default accounts: `admin` (password: `admin123`), `player` (password: `player123`)

**sessions**
- Tracks active login sessions
- Session keys are 32-byte hex strings returned in `AnsLogin` packet
- Auto-expire based on `expires_at` timestamp

**characters**
- Character data per account (supports multiple characters)
- Position stored as floats (X, Y, Z coordinates)
- Soft delete via `deleted_at` timestamp

**character_stats**
- Character attribute points (STR, DEX, INT, VIT, LUK)
- Unallocated stat points

**inventory**
- Character item storage
- Supports stacking (`quantity`), equipment status, and enchantment levels

## Default Test Accounts

Created automatically on first migration:

| Username | Password   | Type  | GM Level |
|----------|------------|-------|----------|
| admin    | admin123   | GM    | 99       |
| player   | player123  | Player| 0        |

**⚠️ WARNING:** Change these passwords before deploying to production!

## Schema Design Notes

### Timestamps
- All timestamps use Unix epoch (seconds since 1970-01-01)
- SQLite: `INTEGER`
- MySQL: `BIGINT UNSIGNED`

### Character Deletion
- Characters are **soft deleted** (set `deleted_at` timestamp)
- Allows character recovery and prevents ID reuse
- Query active characters: `WHERE deleted_at IS NULL`

### Session Management
- Sessions expire based on `expires_at` field
- `last_activity` used for idle timeout detection
- Old sessions should be cleaned up periodically (cron job recommended)

## Adding New Migrations

When adding new migrations, follow this naming convention:

```
XXX_description.sql          (SQLite version)
XXX_description_mysql.sql    (MySQL version)
```

Where `XXX` is a 3-digit number (e.g., `002`, `003`, etc.)

Example:
```
002_add_guild_system.sql
002_add_guild_system_mysql.sql
```

## Rollback

To rollback migrations manually:

```bash
# SQLite
rm ragnoria.db
sqlite3 ragnoria.db < migrations/001_initial_schema.sql

# MySQL
mysql -u root -p -e "DROP DATABASE ragnoria; CREATE DATABASE ragnoria CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"
mysql -u root -p ragnoria < migrations/001_initial_schema_mysql.sql
```

For automated rollback, consider using `sqlx` migrations with down files.
