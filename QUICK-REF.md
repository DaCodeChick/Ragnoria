# Quick Reference - Ragnoria Development

## 🎯 Current Status
- ✅ Project structure complete
- ✅ Packet analyzer tool ready
- ✅ Database migrations ready
- ✅ RMI parser implemented
- ⚠️ **BLOCKED:** Need packet captures to extract real message IDs

## 🚀 Quick Commands

### Build & Run
```bash
cargo build --workspace              # Build everything
cargo check --workspace              # Fast compile check
cargo test --workspace               # Run all tests

cargo run -p ro2-login               # Login server (7101)
cargo run -p ro2-lobby               # Lobby server (7201)
cargo run -p ro2-world               # World server (7401)

RUST_LOG=debug cargo run -p ro2-login  # With debug logging
```

### Packet Analysis
```bash
# Analyze hex dump file
cargo run --bin packet-analyzer -- file docs/captures/login_flow_hex.txt

# Analyze hex string directly
cargo run --bin packet-analyzer -- hex --data "50524F55..."

# Interactive mode (paste hex)
cargo run --bin packet-analyzer -- interactive
```

### Database
```bash
# Create database and run migrations (SQLite)
sqlite3 ragnoria.db < migrations/001_initial_schema.sql

# View accounts
sqlite3 ragnoria.db "SELECT * FROM accounts;"

# Test login (bcrypt verify)
sqlite3 ragnoria.db "SELECT username FROM accounts WHERE username='admin';"
```

### Testing
```bash
# Test server connectivity
echo "test" | nc localhost 7101

# Monitor all RO2 traffic
tcpdump -i any -w capture.pcap 'port 7101 or port 7201 or port 7401'
```

## 📁 Important Files

### Configuration
- `Cargo.toml` - Workspace dependencies
- `.env.example` - Environment template
- `.gitignore` - Excludes captures and DB files

### Code - Common Library
- `crates/ro2-common/src/protocol/mod.rs` - **MessageType enum** (update IDs here!)
- `crates/ro2-common/src/packet/parser.rs` - RmiMessage parser
- `crates/ro2-common/src/database/` - Database models and queries

### Code - Servers
- `crates/ro2-login/src/main.rs` - Login server entry point
- `crates/ro2-login/src/handlers/mod.rs` - Login handlers (implement here!)
- `crates/ro2-lobby/src/` - Lobby server
- `crates/ro2-world/src/` - World server

### Documentation
- `README.md` - Project overview
- `PROGRESS.md` - Current status and next steps
- `QUICKSTART.md` - Getting started guide
- `docs/CAPTURE-GUIDE.md` - **How to capture packets** ⭐
- `docs/captures/EXAMPLE-ANALYSIS.md` - Packet analysis walkthrough
- `docs/protocol/RFC-RO2-PROTOCOL.md` - Protocol specification
- `docs/protocol/appendices/message-catalog.md` - All 660+ messages

### Database
- `migrations/001_initial_schema.sql` - SQLite schema
- `migrations/001_initial_schema_mysql.sql` - MySQL schema
- `migrations/README.md` - Migration guide

## 📊 Project Stats

- **Total Crates:** 5 (ro2-common, ro2-login, ro2-lobby, ro2-world, packet-analyzer)
- **Message Types Identified:** 660+ (from Ghidra analysis)
- **Message IDs Mapped:** 0 (need packet captures!)
- **Servers Implemented:** 3 (all echo mode, need handlers)
- **Database Tables:** 5 (accounts, sessions, characters, character_stats, inventory)

## 🔥 Critical Next Step: Packet Capture

**You cannot proceed without capturing real traffic!**

### Capture Workflow (5 minutes)

1. **Start Wireshark**
   ```
   Filter: tcp port 7101 or tcp port 7201 or tcp port 7401
   ```

2. **Launch RO2 Client**
   - Enter credentials
   - Click login
   - Wait for character select screen

3. **Stop Capture**
   - File → Export Packet Dissections → As Plain Text
   - Enable: Packet bytes (hex dump)
   - Save to: `docs/captures/login_flow_hex.txt`

4. **Analyze**
   ```bash
   cargo run --bin packet-analyzer -- file docs/captures/login_flow_hex.txt
   ```

5. **Update Code**
   - Note the message IDs in output
   - Update `MessageType` enum in `crates/ro2-common/src/protocol/mod.rs`
   - Example: `ReqLogin = 0x0123` (replace 0x0001 with real value)

### Expected First Packets

| Packet | Direction | Port | Expected Message ID Range |
|--------|-----------|------|---------------------------|
| ReqLogin | C→S | 7101 | 0x0100 - 0x01FF |
| AnsLogin | S→C | 7101 | 0x0100 - 0x01FF |
| ReqServerStatus | C→S | 7101 | 0x0100 - 0x01FF |
| AckServerStatus | S→C | 7101 | 0x0100 - 0x01FF |

## 🎓 Learning Resources

### Understanding ProudNet Protocol
- Read: `docs/protocol/RFC-RO2-PROTOCOL.md` section 4 (Message Types)
- Read: `docs/captures/EXAMPLE-ANALYSIS.md` (full walkthrough)
- Reference: `docs/ghidra-findings.md` (raw reverse engineering data)

### Rust Best Practices for This Project
- Use `anyhow::Result` for error handling
- Use `tracing::info!()` / `debug!()` for logging
- Prefer `bytes::Bytes` for zero-copy packet handling
- Use `sqlx::query!()` macro for compile-time SQL verification

## 🐛 Common Issues

### "Message ID not recognized"
→ You're using placeholder IDs. Capture real traffic and update the enum.

### "Database error: no such table"
→ Run migrations: `sqlite3 ragnoria.db < migrations/001_initial_schema.sql`

### "Connection refused" when testing
→ Make sure server is running: `cargo run -p ro2-login`

### "Wireshark shows no packets"
→ Check filter syntax: `tcp.port == 7101 || tcp.port == 7201 || tcp.port == 7401`

### "Cannot parse packet: insufficient data"
→ Packet may be fragmented. In Wireshark: Follow TCP Stream → reassemble

## 📞 Getting Help

### Check These First
1. `PROGRESS.md` - Current status and known issues
2. `docs/CAPTURE-GUIDE.md` - Packet capture troubleshooting
3. `.opencode/AGENTS.md` - Project conventions and guidelines

### Debug Logging
```bash
# Enable all debug output
RUST_LOG=debug cargo run -p ro2-login

# Enable specific module
RUST_LOG=ro2_login::handlers=trace cargo run -p ro2-login

# Capture logs to file
RUST_LOG=debug cargo run -p ro2-login 2>&1 | tee server.log
```

## 🎉 Test Accounts

| Username | Password  | Type | GM Level | Account ID |
|----------|-----------|------|----------|------------|
| admin    | admin123  | GM   | 99       | 1          |
| player   | player123 | User | 0        | 2          |

Passwords are bcrypt hashed in database.

---

**Last Updated:** Session 2 (Packet Analysis Infrastructure)  
**Next Milestone:** Extract message IDs from captures  
**Estimated Time to Next Milestone:** 10-30 minutes (capture + analysis)
