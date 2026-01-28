# Ragnoria Development Progress

## Session 4 Completion Summary (Latest)

### What Was Accomplished

✅ **Deep Ghidra Analysis - Function Renaming**
- Successfully renamed 7 functions using Ghidra MCP server:
  - `FUN_00995900` → `UI_IsDialogOpen` (checks if dialog window is visible)
  - `FUN_006ae6f0` → `GetItemCount` (retrieves item count by ID)
  - `FUN_006aec70` → `InitiateServerConnection` (starts connection process)
  - `FUN_0044b110` → `VectorLength` (calculates 3D vector magnitude)
  - `FUN_00464c40` → `ReleasePlayerReference` (decrements player ref count)
  - `FUN_004d8bd0` → `CleanupPlayerIterator` (iterator cleanup)
  - `FUN_004d8400` → `AdvancePlayerIterator` (move to next player)

✅ **Main Game Loop Analysis**
- Traced complete execution flow from `WinMain @ 0x00A502F0`
- Documented `MainGameLoopUpdate @ 0x00A4C300` (750-line analysis document)
- Identified 10-phase game loop execution:
  1. Time Management (delta time calculation)
  2. Frame Rate Adjustment (High/Medium/Normal tiers)
  3. Timer Updates (3 separate timer instances)
  4. Network Update Phase 1 (pre-game-logic, receive messages)
  5. Game State Update (process game logic)
  6. Renderer Update (Gamebryo engine)
  7. Sound Manager Update (FMOD system)
  8. Network Update Phase 2 (post-game-logic, send messages)
  9. Shutdown Check (global flag mechanism)
  10. Rendering (frame output)

✅ **Network Processing Architecture**
- **Two-Phase Network Model Discovered:**
  - **Phase 1 (vtable+0x24)**: Receives and queues incoming messages BEFORE game logic
  - **Phase 2 (vtable+0x28)**: Sends outgoing messages AFTER game logic completes
  - This allows game state to react to received messages in the same frame
- **NetworkManager Global:** `g_NetworkManager_ProudNetClient @ 0x015B53AC`
- **GetNetworkManager Function:** `@ 0x00A30660` (80+ references in codebase)

✅ **Frame Rate Adaptation System**
- Three performance tiers based on `GameStateManager` performance metric:
  - **High**: Best performance, highest FPS target
  - **Medium**: Moderate performance, mid FPS target  
  - **Normal**: Low performance, minimum FPS target
- Dynamic switching based on real-time performance monitoring
- Performance thresholds at `DOUBLE_01358008` and `DOUBLE_0135b6d8`

✅ **Timer Management System**
- Three timer instances identified:
  - **Timer 0**: Main game timer (frame deltas)
  - **Timer 1**: Secondary timer (frame-based, can be paused)
  - **Timer 2**: Tertiary timer (alternate, gets reset)
- Timer structure (partial):
  ```c
  struct GameTimer {
      float targetFrameRate;      // +0x08
      float currentDelta;         // +0x10
      char isPaused;              // +0x14
  };
  ```

✅ **Critical Globals Mapped**
| Variable | Address | Type | Description |
|----------|---------|------|-------------|
| `g_NetworkManager_ProudNetClient` | 0x015B53AC | `void*` | ProudNet network manager |
| `g_GameStateManager` | 0x015C9088 | `void*` | Game state manager |
| `g_RendererInstance` | 0x015D7708 | `void*` | Gamebryo renderer |
| `g_UIManager` | 0x015B17A8 | `void*` | UI manager |
| `g_ShutdownRequested` | 0x015A1C10 | `bool` | Shutdown flag |
| `PTR_FUN_015a5244` | 0x015A5244 | `FatalErrorHandler*` | Fatal error callback |

✅ **ProudNet Strings Discovered**
- Found 31 ProudNet-related strings in binary:
  - `"Proud::CNetClientWorker::ProcessMessage_ProudNetLayer"` @ 0x01458560
  - `"Proud::CNetCoreImpl::Send_SecureLayer"` @ 0x01457b50
  - `"Proud::CNetCoreImpl::ProcessMessage_Encrypted"` @ 0x01457c90
  - `"Proud::CFastSocket::Connect"` @ 0x014599ec
  - `"Proud::CFastSocket::IssueRecv"` @ 0x01459a24
  - `"Proud::CFastSocket::IssueSend"` @ 0x01459b3c
  - Plus 25 more related to heap management, critical sections, sockets

✅ **Documentation Created**
- **`docs/ghidra-findings/GAME-LOOP-ANALYSIS.md`** (750 lines)
  - Complete MainGameLoopUpdate flow diagram
  - Network processing model (two-phase design)
  - Frame rate adaptation algorithm
  - Timer management details
  - Shutdown sequence documentation
  - Structure definitions (GameStateManager, GameTimer, Renderer, etc.)
  - Critical functions table (50+ functions)
  - Implementation notes for Rust server

✅ **Manager Singleton Pattern Identified**
All critical subsystems use singleton getters:
- `GetNetworkManager()` - Returns network manager
- `GetGameStateManager()` - Returns game state
- `GetGameTimersInstance()` - Returns timer manager
- `GetRendererInstance()` - Returns renderer
- `GetSoundManager()` - Returns sound system (nullable)
- `GetGameConfigurationManager()` - Returns config manager
- `GetLocalizationManager()` - Returns localization system

### Key Discoveries

1. **Message Processing Model**
   ```
   Receive Messages → Queue → Process Game Logic → Send Responses
   ```
   This two-phase design ensures responses reflect updated game state.

2. **Frame Timing**
   - Game uses `GetCurrentGameTime()` returning 80-bit float (float10)
   - Delta time is clamped to prevent large jumps during lag
   - Sleep(0) yields CPU but allows immediate re-scheduling

3. **Input Handling**
   - Special routing for input messages when `g_InputSystemEnabled == false`
   - Key messages (WM_KEYDOWN through WM_KEYLAST) get forwarded to `g_InputWindow`
   - TranslateMessage/DispatchMessage handle standard Windows messages

4. **Shutdown Mechanism**
   - Global flag `g_ShutdownRequested` triggers shutdown
   - `HandleGameShutdownRequest()` initiates cleanup
   - `CleanupNetworkAndResources()` called at exit (atexit registered)

### Build Status
```
✅ cargo check --workspace  (clean)
✅ cargo build --workspace  (clean)
✅ git commit successful     (8aab852)
```

### Git Commits

- **Commit 1 (b4ccb5f):** Initial project scaffold
- **Commit 2 (5d14e19):** Packet analysis tooling, migrations, RMI parser
- **Commit 3 (5cddd6a):** Add progress tracking document
- **Commit 4 (4c2cc5c):** Add quick reference card
- **Commit 5 (13d8ef4):** Modernize to Rust 2024 edition
- **Commit 6 (6cde17a):** Update PROGRESS.md with session 3 summary
- **Commit 7 (6d1af18):** Add Ghidra analysis guidelines to AGENTS.md
- **Commit 8 (48c4585):** Add comprehensive WinMain traversal analysis
- **Commit 9 (8aab852):** Add game loop and network processing analysis ← YOU ARE HERE

### Next Steps (Priority Order)

#### 🔴 Critical: Continue Ghidra Analysis

1. **Find Network VTable Functions** (HIGH PRIORITY)
   - Disassemble NetworkManager vtable to get function pointers at offsets 0x24 and 0x28
   - Decompile and analyze both network update functions
   - Trace to message dispatch handlers
   - Document complete message processing flow

2. **Analyze GameStateManager_Update** (HIGH PRIORITY)
   - Decompile and trace game state update flow
   - Identify message processing callbacks
   - Find RMI message handlers
   - Map state machine transitions

3. **ProudNet Message Handlers** (HIGH PRIORITY)
   - Find and analyze handlers for system messages:
     - `0x0B` - Version check
     - `0x1F` - TCP connection established
     - `0x20` - TCP disconnection
   - Determine message dispatch table structure
   - Document handler signatures

4. **Localization String Table** (MEDIUM PRIORITY)
   - Analyze `LocalizationManager_GetString` implementation
   - Find string table location in binary
   - Extract string IDs and their English text
   - Create reference table

5. **Rename Remaining Functions** (ONGOING)
   - Continue systematic renaming following AGENTS.md guidelines
   - Focus on ProudNet and network-related functions
   - Document function parameters and return types

#### 🟡 Code Implementation

6. **Update MessageType Enum** (MEDIUM PRIORITY)
   - Add system message IDs discovered: `0x0B`, `0x1F`, `0x20`
   - Add game RMI message IDs when found
   - Update `crates/ro2-common/src/protocol/mod.rs`

7. **Implement Network Processing Model** (MEDIUM PRIORITY)
   - Design two-phase network update in Rust
   - Create message queue system
   - Implement pre/post game logic hooks

8. **Crypto Analysis** (LOW PRIORITY - after finding handlers)
   - Analyze ProudNet encryption functions if referenced
   - Determine AES mode and padding
   - Implement in `crates/ro2-common/src/crypto/mod.rs`

### Resources for Next Session

- **Ghidra MCP Server:** Connected to Rag2.exe
- **Analysis Documents:**
  - `docs/ghidra-findings/WINMAIN-ANALYSIS.md` (600 lines)
  - `docs/ghidra-findings/GAME-LOOP-ANALYSIS.md` (750 lines)
- **Naming Conventions:** `.opencode/AGENTS.md`
- **ProudNet Strings:** 31 identified at addresses 0x01450110+
- **Network Manager Global:** 0x015B53AC

### Important Notes

⚠️ **Network VTable Analysis Pending**  
The vtable function pointers at offsets 0x24 and 0x28 need to be extracted by disassembling the vtable structure. This is the next critical step to understand message processing.

⚠️ **Message Dispatch Table Not Found Yet**  
Need to locate the ProudNet message dispatch table that maps message IDs to handler functions.

⚠️ **Global Variable Addresses Partially Confirmed**  
Some globals from WinMain analysis have confirmed addresses, others (timer-related, frame rate constants) need confirmation through further analysis.

### Questions Answered This Session

Q: How does the client process network messages?  
A: Two-phase update: (1) Receive/queue before game logic, (2) Send/flush after game logic. This allows same-frame reactions.

Q: What is the main game loop structure?  
A: 10-phase execution: time management → frame rate adaptation → timers → network phase 1 → game logic → renderer → sound → network phase 2 → shutdown check → render.

Q: How are managers accessed throughout the codebase?  
A: Singleton pattern with getter functions (GetNetworkManager, GetGameStateManager, etc.) returning global pointers.

Q: What is the frame rate system?  
A: Dynamic 3-tier system (High/Medium/Normal) that adjusts FPS targets based on real-time performance metrics from GameStateManager.

---

## Session 3 Completion Summary

### What Was Accomplished

✅ **Rust 2024 Edition Upgrade**
- Upgraded all crates to Rust 2024 edition
- Added `rust-version = "1.93"` requirement
- Fixed `gen` reserved keyword conflict in crypto module
- All code now uses modern Rust patterns

✅ **Dependency Modernization**
- **Replaced unmaintained crates:**
  - `bincode` → `postcard` 1.1.3 (bincode abandoned after developer incident)
  - `dotenv` → `dotenvy` 0.15.7 (actively maintained fork)
- **Updated to latest versions:**
  - `tokio` 1.49.0 (async runtime)
  - `bcrypt` 0.18.0 (password hashing)
  - `thiserror` 2.0 (error handling)
  - `bytes` 1.9.0 (buffer management)
  - `config` 0.15.19 (configuration)

✅ **Code Quality Improvements**
- Fixed Rust 2024 reserved keyword issues
- Cleaned up unused imports and variables
- Fixed test suite (all tests pass)
- Zero compilation errors or warnings (except expected dead code)

✅ **Documentation**
- Created `docs/RUST-2024-MIGRATION.md` with full migration guide
- Documented why each dependency was chosen
- Added performance notes and breaking changes guide

### Build Status
```
✅ cargo check --workspace  (0.47s)
✅ cargo build --workspace  (11.82s)
✅ cargo test --workspace   (all tests pass)
```

---

## Session 2 Completion Summary

### What Was Accomplished

✅ **Packet Analysis Infrastructure**
- Created `packet-analyzer` utility binary (crates/packet-analyzer/)
  - Parses Wireshark hex dumps to extract message IDs
  - Analyzes payload structure and detects encryption
  - Calculates entropy to identify plaintext vs encrypted data
  - CLI interface: file parsing, hex string parsing, interactive mode

✅ **RMI Message Parser**
- Implemented `RmiMessage` struct in `crates/ro2-common/src/packet/parser.rs`
  - Parses ProudNet RMI packet headers (16 bytes)
  - Extracts: magic, length, message_id, flags, sequence, payload
  - Validates packet integrity
- Implemented `RmiMessageBuilder` for constructing responses
  - Fluent API for building packets
  - Helper methods: `write_string()`, `write_u32()`, `write_u16()`, `write_u8()`

✅ **Database Migrations**
- Created SQLite migration: `migrations/001_initial_schema.sql`
- Created MySQL migration: `migrations/001_initial_schema_mysql.sql`
- Tables: accounts, sessions, characters, character_stats, inventory
- Default test accounts:
  - `admin` / `admin123` (GM level 99)
  - `player` / `player123` (normal user)
- Comprehensive documentation in `migrations/README.md`

✅ **Dependencies Added**
- `chrono` - For timestamp handling in database queries
- `bcrypt` - For password hashing (test accounts use bcrypt)
- `clap` - CLI argument parsing for packet analyzer
- `hex` - Hex string encoding/decoding

✅ **Documentation Created**
1. **`docs/CAPTURE-GUIDE.md`**
   - Step-by-step Wireshark capture instructions
   - Filters for RO2 traffic (ports 7101, 7201, 7401)
   - Export formats (pcapng, hex dumps)
   - Troubleshooting common capture issues

2. **`docs/captures/EXAMPLE-ANALYSIS.md`**
   - Complete walkthrough of analyzing ReqLogin/AnsLogin packets
   - Annotated hex dumps with field explanations
   - Payload structure documentation (C-like structs)
   - Code examples showing how to implement handlers

3. **`docs/captures/README.md`**
   - How to use the captures directory
   - Security warnings about sensitive data

4. **`migrations/README.md`**
   - Database setup instructions
   - Schema overview
   - Migration management guide

✅ **Enhanced .gitignore**
- Excluded packet captures (*.pcapng, *.pcap)
- Excluded database files (*.db, *.db-*)
- Excluded .env files

✅ **Code Quality**
- All code compiles with `cargo build --workspace`
- Unit tests for RmiMessage parsing
- Proper error handling with `anyhow::Result`

### Project Status

**Ready for:** Packet capture and message ID extraction

**Current Workflow:**
1. Capture RO2 traffic with Wireshark → `docs/captures/login_flow.pcapng`
2. Export as hex dump → `docs/captures/login_flow_hex.txt`
3. Analyze: `cargo run --bin packet-analyzer -- file docs/captures/login_flow_hex.txt`
4. Extract message IDs from output
5. Update `MessageType` enum in `crates/ro2-common/src/protocol/mod.rs`
6. Implement payload parsers based on analysis
7. Test with real RO2 client

### File Structure Overview

```
Ragnoria/
├── crates/
│   ├── packet-analyzer/       ← NEW: Hex dump analysis tool
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   ├── ro2-common/
│   │   └── src/
│   │       ├── packet/
│   │       │   ├── mod.rs
│   │       │   └── parser.rs  ← NEW: RMI message parser
│   │       ├── protocol/
│   │       │   └── mod.rs     ← UPDATED: Added from_id(), to_id()
│   │       └── database/
│   │           └── queries.rs ← UPDATED: Fixed imports
│   ├── ro2-login/
│   ├── ro2-lobby/
│   └── ro2-world/
├── docs/
│   ├── CAPTURE-GUIDE.md       ← NEW: Wireshark guide
│   └── captures/              ← NEW: Capture storage
│       ├── README.md
│       └── EXAMPLE-ANALYSIS.md ← NEW: Full analysis walkthrough
├── migrations/                 ← NEW: Database schemas
│   ├── README.md
│   ├── 001_initial_schema.sql
│   └── 001_initial_schema_mysql.sql
└── .gitignore                 ← UPDATED: Exclude sensitive files
```

### Git Commits

- **Commit 1 (b4ccb5f):** Initial project scaffold
- **Commit 2 (5d14e19):** Packet analysis tooling, migrations, RMI parser
- **Commit 3 (5cddd6a):** Add progress tracking document
- **Commit 4 (4c2cc5c):** Add quick reference card
- **Commit 5 (13d8ef4):** Modernize to Rust 2024 edition ← YOU ARE HERE

### Next Steps (Priority Order)

#### 🔴 Critical: Packet Capture Required
Before any further implementation, you must capture real traffic:

```bash
# 1. Start Wireshark with filter: tcp port 7101 or tcp port 7201 or tcp port 7401
# 2. Launch RO2 client and attempt login
# 3. Stop capture and export as hex dump
# 4. Run analyzer:
cargo run --bin packet-analyzer -- file docs/captures/login_flow_hex.txt
```

#### After Packet Capture

1. **Map Message IDs** (HIGH PRIORITY)
   - Update `MessageType` enum with real IDs from captures
   - Document ID mappings in `docs/captures/message_id_mapping.md`

2. **Implement Payload Parsers** (HIGH PRIORITY)
   - Create `crates/ro2-common/src/protocol/messages.rs`
   - Implement `ReqLogin::parse()` and `AnsLogin::serialize()`
   - Add other message structs as discovered

3. **Database Setup** (MEDIUM PRIORITY)
   ```bash
   sqlite3 ragnoria.db < migrations/001_initial_schema.sql
   ```

4. **Implement Login Handler** (HIGH PRIORITY)
   - Update `crates/ro2-login/src/handlers/mod.rs`
   - Integrate database queries
   - Validate credentials with bcrypt
   - Generate session keys

5. **Test with Real Client** (HIGH PRIORITY)
   - Point RO2 client to local server
   - Monitor logs: `RUST_LOG=debug cargo run -p ro2-login`
   - Validate login flow

### Key Commands

```bash
# Build everything
cargo build --workspace

# Run servers
cargo run -p ro2-login          # Port 7101
cargo run -p ro2-lobby          # Port 7201
cargo run -p ro2-world          # Port 7401

# Analyze packet captures
cargo run --bin packet-analyzer -- file docs/captures/login_flow_hex.txt
cargo run --bin packet-analyzer -- hex --data "50524F55..."

# Database setup
sqlite3 ragnoria.db < migrations/001_initial_schema.sql

# Test connectivity
echo "test" | nc localhost 7101
```

### Important Notes

⚠️ **Message IDs are still placeholders!**  
The `MessageType` enum uses dummy values (0x0001, 0x0002, etc.). These MUST be replaced with real values from packet captures before the servers will work with the real client.

⚠️ **Encryption not implemented**  
The `CryptoHandler` functions are stubs. Based on packet capture analysis, we'll determine if encryption is needed and implement accordingly.

⚠️ **Database not connected**  
The servers don't query the database yet. After implementing message parsers, integrate `AccountQueries` and `SessionQueries`.

### Questions Answered This Session

Q: How do we extract message IDs from the client?  
A: Capture network traffic and parse the hex dumps with our packet analyzer tool.

Q: What database schema should we use?  
A: Created comprehensive schemas with accounts, sessions, characters, stats, and inventory tables.

Q: How do we parse ProudNet packets?  
A: Implemented `RmiMessage` parser that handles the 16-byte header + variable payload.

Q: How should we store captures securely?  
A: Added captures to .gitignore and documented security best practices.

### Resources for Next Session

- **Ghidra MCP Server:** Available for deeper binary analysis if needed
- **Test Accounts:** admin/admin123, player/player123 (pre-seeded in database)
- **ProudNet Protocol Docs:** `docs/protocol/RFC-RO2-PROTOCOL.md`
- **Message Catalog:** `docs/protocol/appendices/message-catalog.md` (660+ messages)

---

**Current State:** Infrastructure complete, ready for packet capture phase  
**Blocking Issue:** Need real packet captures to extract message IDs  
**Next Action:** Follow `docs/CAPTURE-GUIDE.md` to capture login traffic
