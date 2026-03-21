# Quick Start Guide

## Prerequisites

- Rust 1.70+ (`rustup` recommended)
- SQLite 3 (or MySQL for production)
- Wireshark (for packet capture)
- Ghidra (for further reverse engineering)

## Setup

1. **Clone the repository**
```bash
cd /path/to/Ragnoria
```

2. **Create environment file**
```bash
cp .env.example .env
# Edit .env if needed
```

3. **Build the project**
```bash
cargo build --workspace
```

4. **Run database migrations** (TODO - not yet implemented)
```bash
sqlx migrate run
```

## Running the Servers

### Login Server (Port 7101)
```bash
cargo run -p ro2-login
```

### Lobby Server (Port 7201)
```bash
cargo run -p ro2-lobby
```

### World Server (Port 7401)
```bash
cargo run -p ro2-world
```

## Testing

Currently, the servers are echo servers for testing TCP connectivity.

```bash
# Test login server
echo "test" | nc localhost 7101

# Test lobby server
echo "test" | nc localhost 7201

# Test world server
echo "test" | nc localhost 7401
```

## Next Steps

### 1. Capture Network Traffic

Run the actual RO2 client and capture login traffic:

```bash
# Start Wireshark
wireshark

# Filter: tcp.port == 7101 or tcp.port == 7201 or tcp.port == 7401
# Capture traffic during client login
```

Save captures to `docs/captures/` directory.

### 2. Analyze Packet Structure

Use Wireshark to determine:
- Message ID values (numeric)
- Packet payload format
- Encryption handshake sequence
- Login message structure

### 3. Implement Login Flow

Update `crates/ro2-login/src/handlers/mod.rs`:
- Parse `ReqLogin` packet
- Validate credentials
- Generate session key
- Send `AnsLogin` response

### 4. Database Schema

Create migrations in `migrations/`:
```sql
-- migrations/20260127_initial.sql
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    is_banned INTEGER DEFAULT 0
);

CREATE TABLE sessions (
    id INTEGER PRIMARY KEY,
    account_id INTEGER NOT NULL,
    session_key TEXT UNIQUE NOT NULL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    is_active INTEGER DEFAULT 1,
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);
```

## Development Workflow

1. **Analyze with Ghidra** - Study client message handlers
2. **Capture with Wireshark** - Record real client traffic
3. **Document in RFC** - Update protocol documentation
4. **Implement in Rust** - Build server handler
5. **Test with client** - Validate against real RO2 client

## Debugging

Enable debug logging:
```bash
RUST_LOG=debug cargo run -p ro2-login
```

Enable trace logging for specific module:
```bash
RUST_LOG=ro2_common::packet=trace cargo run -p ro2-login
```

## Common Issues

### Cargo build fails
```bash
# Update Rust toolchain
rustup update stable

# Clean and rebuild
cargo clean
cargo build --workspace
```

### Port already in use
```bash
# Check what's using the port
lsof -i :7101
# or
netstat -tulpn | grep 7101

# Kill the process or change port in .env
```

### Database errors
```bash
# Verify SQLite is installed
sqlite3 --version

# Check database file permissions
ls -la ragnoria.db
```

## Project Structure Reference

```
ragnoria/
├── .opencode/AGENTS.md              # AI development guidelines
├── docs/
│   ├── protocol/
│   │   ├── RFC-RO2-PROTOCOL.md      # Protocol specification
│   │   └── appendices/
│   │       └── message-catalog.md   # All 660+ messages
│   └── ghidra-findings.md           # Reverse engineering notes
├── crates/
│   ├── ro2-common/                  # Shared library
│   │   ├── src/
│   │   │   ├── protocol/            # ProudNet RMI
│   │   │   ├── packet/              # Packet structures
│   │   │   ├── crypto/              # AES/RSA
│   │   │   └── database/            # Models
│   ├── ro2-login/                   # Login server
│   ├── ro2-lobby/                   # Lobby server
│   └── ro2-world/                   # World server
└── migrations/                      # Database migrations
```

## Resources

- [RFC Protocol Spec](docs/protocol/RFC-RO2-PROTOCOL.md)
- [Ghidra Findings](docs/ghidra-findings.md)
- [Message Catalog](docs/protocol/appendices/message-catalog.md)
- [AI Agent Guidelines](.opencode/AGENTS.md)

## Getting Help

1. Check the documentation in `docs/`
2. Review Ghidra findings for protocol details
3. Examine packet captures with Wireshark
4. Consult the RFC for message specifications

---

**Remember:** This is a reverse engineering project for educational purposes. Always analyze the real client to validate your implementation!
