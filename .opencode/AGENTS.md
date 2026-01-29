# Ragnoria Project - AI Agent Guidelines

## Project Overview

**Name:** Ragnoria  
**Type:** Ragnarok Online 2 Server Emulator  
**Language:** Rust  
**Purpose:** Educational reverse engineering and game networking study

## Project Goals

1. **Primary Goal**: Create a functional RO2 server emulator that replicates login flow
2. **Learning Goal**: Understand real-world MMO network protocols and ProudNet RMI
3. **Research Goal**: Document the RO2 protocol through reverse engineering

## Architecture Principles

### Workspace Structure
- **Cargo workspace** with 4 crates: `ro2-common`, `ro2-login`, `ro2-lobby`, `ro2-world`
- **Database-agnostic**: Use `sqlx` with SQLite initially, MySQL later
- **Modular design**: Protocol, crypto, and database logic in `ro2-common`

### Technology Stack
- **Rust** for memory safety and network performance
- **Tokio** for async runtime
- **sqlx** for database (SQLite → MySQL migration path)
- **Ghidra** for reverse engineering client
- **Wireshark** for packet capture analysis

## Critical Conventions

### Code Style
- Follow Rust naming conventions (snake_case for functions/variables)
- Use `anyhow` for application errors, `thiserror` for library errors
- Implement `tracing` for structured logging (not `println!`)
- Document all public APIs with doc comments

### Protocol Implementation
- All packet structures MUST match Ghidra analysis exactly
- Message IDs MUST be validated against real client traffic
- Never hardcode "magic numbers" - use named constants
- Always use little-endian byte order (client is x86)

### Database
- Use `sqlx` compile-time checked queries
- Support both SQLite and MySQL via feature flags
- Store credentials with proper hashing (bcrypt/argon2)
- Design schema for eventual horizontal scaling

### Security
- Never log passwords or session keys
- Implement rate limiting on login attempts
- Validate all client input before processing
- Use constant-time comparison for password checks

## Project Phases

### Phase 1: Foundation (CURRENT)
- [x] Create Cargo workspace structure
- [x] Document protocol in RFC format (`docs/protocol/RFC-RO2-PROTOCOL.md`)
- [ ] Implement basic packet parsing
- [ ] Set up SQLite database with migrations

### Phase 2: Login Server
- [ ] TCP listener on port 7101
- [ ] Handle `ReqLogin` / `AnsLogin` messages
- [ ] Authenticate against database
- [ ] Generate and distribute session keys
- [ ] Handle `ReqServerStatus` / `AckServerStatus`

### Phase 3: Lobby Server
- [ ] TCP listener on port 7201
- [ ] Validate session keys from login server
- [ ] Implement channel list functionality
- [ ] Handle character selection (read-only for PoC)

### Phase 4: Protocol Refinement
- [ ] Capture real client traffic with Wireshark
- [ ] Map message IDs from packet captures
- [ ] Implement AES/RSA encryption
- [ ] Validate against actual RO2 client

### Phase 5: World Server (Future)
- [ ] TCP listener on port 7401
- [ ] Basic player spawn
- [ ] Movement synchronization
- [ ] Chat system

## Common Pitfalls to Avoid

### DON'T
- **Don't guess packet structures** - always verify with Ghidra or captures
- **Don't skip error handling** - this is networking code, things will fail
- **Don't use `unwrap()`** in production code - use `?` or `expect()` with context
- **Don't implement encryption first** - validate protocol structure with plaintext
- **Don't hardcode server addresses** - use configuration files
- **Don't create files without asking** - especially README/docs (user preference)

### DO
- **Do verify against Ghidra** before implementing structures
- **Do test with real client** whenever possible
- **Do document discoveries** in RFC-style protocol doc
- **Do ask clarifying questions** before making assumptions
- **Do use incremental development** - one message type at a time
- **Do capture network traffic** to validate implementation

## Key Files to Reference

### Documentation
- `.opencode/AGENTS.md` - This file (AI guidelines)
- `docs/protocol/RFC-RO2-PROTOCOL.md` - Protocol specification (RFC format)
- `docs/ghidra-findings.md` - Raw Ghidra analysis notes

### Source Code
- `crates/ro2-common/src/protocol/` - ProudNet RMI implementation
- `crates/ro2-common/src/packet/` - Packet structures (must match Ghidra)
- `crates/ro2-common/src/crypto/` - AES/RSA encryption
- `crates/ro2-login/src/handlers/` - Login message handlers

### Configuration
- `Cargo.toml` - Workspace root with shared dependencies
- `migrations/` - sqlx database migrations
- `.env.example` - Configuration template

## Ghidra Integration

**Available via MCP server:**
- Binary: `Rag2.exe` (RO2 client)
- Use Ghidra tools to:
  - Extract structures: `PacketHeader`, `NetworkPacket`, `CompletePacket`
  - List message strings: Search for `Req`, `Ans`, `Nfy`, `Ack` prefixes
  - Find encryption functions: Search for AES/RSA imports
  - Analyze control flow: Understand message dispatching

### Critical Ghidra Analysis Guidelines

**MANDATORY NAMING CONVENTION:**

When analyzing the client in Ghidra, you **MUST** rename every symbol for clarity:

1. **Functions** - Rename from `FUN_00401234` to descriptive names
   - Use purpose-based names: `Initialize_ProudNet`, `Handle_ReqLogin`, `Parse_PacketHeader`
   - Follow PascalCase for client function names (matching C++ style)
   - Example: `FUN_00405678` → `Connect_To_LoginServer`

2. **Function Parameters** - Rename from `param_1`, `param_2`, etc.
   - Use descriptive names: `socket`, `buffer`, `packet_length`, `message_id`
   - Follow snake_case for parameter names
   - Example: `param_1` → `tcp_socket`, `param_2` → `packet_buffer`

3. **Global Variables** - Rename from `DAT_00601234`
   - Use SCREAMING_SNAKE_CASE for globals
   - Include type/purpose hints: `G_CLIENT_VERSION`, `G_SESSION_KEY`, `G_PROUDNET_INSTANCE`
   - Example: `DAT_00603040` → `G_NETWORK_MANAGER`

4. **Local Variables** - Rename from `local_8`, `local_10`, etc.
   - Use descriptive camelCase or snake_case
   - Example: `local_8` → `messageId`, `local_10` → `packetSize`

5. **Structures** - Define and name custom types
   - Use TitleCase for structure names
   - Example: Create `PacketHeader` type, apply to memory regions

**Analysis Workflow:**

1. **Start at WinMain** - Always begin analysis from the entry point
2. **Traverse Call Graph** - Follow function calls depth-first
3. **Rename As You Go** - Don't leave unnamed symbols behind
4. **Document in Comments** - Add Ghidra comments explaining logic
5. **Export Findings** - Document in `docs/ghidra-findings/` (organized by topic)

**Example Ghidra Session:**

```
1. Find WinMain (entry point)
   - Rename: "WinMain" if not already named
   
2. Identify initialization sequence
   - Find ProudNet init → Rename: "Initialize_ProudNetClient"
   - Find network setup → Rename: "Setup_NetworkSockets"
   
3. Find message handlers
   - Locate dispatch table → Rename: "G_MESSAGE_DISPATCH_TABLE"
   - Each handler → Rename: "Handle_ReqLogin", "Handle_AnsServerStatus", etc.
   
4. Extract structures
   - Create types: PacketHeader, NetworkPacket, CompletePacket
   - Apply to all instances in binary
   
5. Find global state
   - Session key storage → "G_SESSION_KEY_BUFFER"
   - Server address → "G_LOGIN_SERVER_ADDRESS"
```

**Naming Patterns:**

| Symbol Type | Pattern | Example |
|-------------|---------|---------|
| Functions | `Verb_Noun` | `Parse_LoginRequest`, `Send_PacketToServer` |
| Methods | `Class_Method` | `ProudNet_Initialize`, `Socket_Connect` |
| Parameters | `snake_case` | `packet_buffer`, `socket_handle`, `message_id` |
| Locals | `camelCase` | `bytesRead`, `isConnected`, `packetType` |
| Globals | `G_SCREAMING_SNAKE` | `G_CLIENT_VERSION`, `G_NETWORK_STATE` |
| Constants | `SCREAMING_SNAKE` | `MAX_PACKET_SIZE`, `LOGIN_SERVER_PORT` |
| Structures | `PascalCase` | `PacketHeader`, `SessionInfo`, `PlayerData` |

**Why This Matters:**

- Makes code review easier when collaborating
- Allows AI to understand context without re-analysis
- Creates exportable documentation
- Facilitates sharing findings with community
- Makes future updates faster (no re-discovering symbols)

**Workflow:**
1. Hypothesize protocol feature from game behavior
2. Search Ghidra for related strings/structures
3. Implement in Rust based on findings
4. Validate with Wireshark capture
5. Test with real client

## Testing Strategy

### Unit Tests
- Test packet serialization/deserialization
- Test crypto functions with known vectors
- Test database queries in isolation

### Integration Tests
- Mock client connections
- Full login flow (client → login → lobby)
- Session key validation

### Client Testing
- Modify client config to point to emulator
- Monitor with Wireshark to compare traffic
- Log all unhandled message types

## Current Status

**Last Updated:** 2026-01-27

**Completed:**
- [x] Project planning and architecture design
- [x] Ghidra analysis of client packet structures
- [x] Protocol reverse engineering from strings
- [x] RFC-format protocol documentation created
- [x] AGENTS.md guidelines document created
- [x] Cargo workspace structure scaffolded

**In Progress:**
- [ ] Implementing common crate packet structures
- [ ] Database schema design

**Blocked:**
- Packet captures not yet available (need to run client)
- Message ID mapping requires real traffic
- Encryption details need deeper Ghidra analysis

## Questions to Ask User

Before implementing features, consider asking:

1. **Database Schema**: "Should I design the initial account/character schema, or do you have specific requirements?"

2. **Encryption Priority**: "Should we implement full AES/RSA from the start, or validate protocol structure with plaintext first?"

3. **Testing Approach**: "Should I create a mock Rust client for testing, or test directly with the real RO2 client?"

4. **ProudNet Depth**: "How deeply should we replicate ProudNet's RMI architecture vs. simplified TCP socket approach?"

5. **Error Handling**: "What level of error recovery do you want? (graceful reconnect, detailed error codes, etc.)"

## Useful Commands

```bash
# Run login server
cargo run -p ro2-login

# Run with logging
RUST_LOG=debug cargo run -p ro2-login

# Database migrations
sqlx migrate run

# Test with SQLite
cargo test --features sqlite

# Test with MySQL (future)
cargo test --features mysql

# Build all crates
cargo build --workspace

# Check without building
cargo check --workspace

# Format code
cargo fmt --all

# Lint
cargo clippy --workspace
```

## Resources

- **Ghidra Docs**: https://ghidra-sre.org/
- **Wireshark Docs**: https://www.wireshark.org/docs/
- **ProudNet Info**: Limited public documentation (proprietary)
- **Rust Networking**: Tokio documentation
- **sqlx Guide**: https://github.com/launchbadge/sqlx

## Communication Style

When working with the AI agent:
- Be specific about what to implement
- Reference Ghidra findings when discussing structures
- Ask to verify against protocol RFC before coding
- Request Wireshark validation after implementation
- Keep scope focused (one message type per session)

## Project Acronyms & Reminders

To save typing and wrist strain, use these shorthand commands:

### **RDS** - Refactor, Despaghettify, Simplify
When you see messy code from discovery/debugging:
- Remove debug println! statements (use tracing instead)
- Clean up unnecessary dependencies
- Simplify control flow
- Remove commented-out code
- Consolidate duplicate logic
- Make code production-ready
- **DO NOT** add research artifacts (PCAP frame numbers, Ghidra addresses, raw analysis notes)
- **DO NOT** over-document implementation details that would clutter the codebase for others
- Keep comments focused on "why" not "what" or "how we discovered this"
- Remove experimental/diagnostic code that served its purpose during discovery

### **DICK** - Do It for Clarity, Knucklehead
When Ghidra symbols aren't renamed:
- Rename ALL functions from `FUN_00401234` to descriptive names
- Rename ALL variables from `param_1`, `local_8` to meaningful names
- Rename ALL globals from `DAT_00601234` to `G_DESCRIPTIVE_NAME`
- Add comments explaining logic
- **Never** leave analysis work half-done
- **CRITICAL:** Follow the ENTIRE call chain - rename ALL called functions too
  - If `Function_A` calls `FUN_00401234`, `FUN_00405678`, and `FUN_0040abcd`, rename ALL of them
  - Recursively follow calls until you hit well-known library functions or previously renamed functions
  - Don't leave "islands" of renamed functions surrounded by `FUN_*` calls
- See "Critical Ghidra Analysis Guidelines" section above for naming conventions

**Usage Examples:**
- User: "RDS this file" → Clean up all debug mess, remove unused code
- User: "DICK this function" → Properly rename all Ghidra symbols in the specified function
- User: "RDS and DICK" → Full cleanup + proper Ghidra renaming pass

---

*This document should be updated as the project evolves and new insights are discovered.*
