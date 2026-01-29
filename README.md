# Ragnoria - Ragnarok Online 2 Server Emulator

A proof-of-concept server emulator for Ragnarok Online 2, built in Rust for educational purposes.

## Project Status

🚧 **Early Development** - Currently implementing login flow

### Completed
- ✅ Project structure and workspace setup
- ✅ Protocol documentation (RFC format)
- ✅ Ghidra reverse engineering analysis
- ✅ Packet structure definitions
- ✅ Basic server scaffolding (Login, Lobby, World)
- ✅ ProudNet encryption (RSA-1024 + AES-128 ECB)
- ✅ Packet capture analysis (PCAP decryption)
- ✅ Login server with working ProudNet handshake
- ✅ Custom launcher GUI (iced framework)
- ✅ Feature flags (client/server separation)

### In Progress
- 🔄 Real client testing with custom launcher
- 🔄 Parameter format discovery for Rag2.exe
- 🔄 Game protocol handlers (post-encryption)

### Planned
- ⏳ Login server authentication flow
- ⏳ Database schema and migrations
- ⏳ Lobby server channel management
- ⏳ Character system and world server

## Architecture

```
┌──────────┐         ┌──────────────┐         ┌──────────┐
│  Client  │◄───────►│ Login Server │◄───────►│ Database │
│ (Rag2.exe)│         │  Port 7101   │         │ (SQLite) │
└──────────┘         └──────────────┘         └──────────┘
     │                      │
     │                      ▼
     │               ┌──────────────┐
     └──────────────►│ Lobby Server │
                     │  Port 7201   │
                     └──────────────┘
                            │
                            ▼
                     ┌──────────────┐
                     │ World Server │
                     │  Port 7401   │
                     └──────────────┘
```

## Technology Stack

- **Language:** Rust (Edition 2021)
- **Async Runtime:** Tokio
- **Database:** SQLx (SQLite → MySQL)
- **Reverse Engineering:** Ghidra SRE
- **Packet Analysis:** Wireshark
- **Networking:** ProudNet RMI protocol

## Documentation

- **[RFC Protocol Specification](docs/protocol/RFC-RO2-PROTOCOL.md)** - Complete protocol documentation
- **[Message Catalog](docs/protocol/appendices/message-catalog.md)** - All 660+ RMI messages
- **[Ghidra Findings](docs/ghidra-findings.md)** - Reverse engineering notes
- **[AI Agent Guidelines](.opencode/AGENTS.md)** - Development guidelines for AI assistance

## Project Structure

```
ragnoria/
├── .opencode/
│   └── AGENTS.md                 # AI agent guidelines
├── docs/
│   ├── protocol/
│   │   ├── RFC-RO2-PROTOCOL.md   # RFC-style protocol spec
│   │   ├── PACKET-CAPTURE-ANALYSIS.md # PCAP analysis
│   │   └── appendices/
│   │       └── message-catalog.md # Complete message list
│   └── ghidra-findings.md        # Reverse engineering notes
├── crates/
│   ├── ro2-common/               # Shared functionality
│   │   ├── src/
│   │   │   ├── protocol/         # ProudNet RMI
│   │   │   ├── packet/           # Packet structures
│   │   │   ├── crypto/           # AES/RSA encryption
│   │   │   └── database/         # Models & queries
│   ├── ro2-login/                # Login server (port 7101)
│   ├── ro2-lobby/                # Lobby server (port 7201)
│   ├── ro2-world/                # World server (port 7401)
│   ├── packet-analyzer/          # PCAP decryption tools
│   │   └── src/bin/
│   │       └── pcap_decrypt.rs   # PCAP decryption tool
│   └── launcher/                 # Custom game launcher GUI
├── migrations/                   # Database migrations
└── Cargo.toml                    # Workspace configuration
```

## Building

```bash
# Build all crates
cargo build --workspace

# Build release version
cargo build --workspace --release

# Build individual components
cargo build -p ro2-login
cargo build -p ro2-lobby
cargo build -p ro2-world
cargo build --bin launcher

# Run with debug logging
RUST_LOG=debug cargo run -p ro2-login
```

## Running

### Using the Custom Launcher

1. **Start the login server:**
   ```bash
   cargo run -p ro2-login
   ```

2. **Launch the custom launcher:**
   ```bash
   cargo run --bin launcher
   ```
   
3. **Configure and launch:**
   - Enter server IP: `127.0.0.1`
   - Enter server port: `7101`
   - Browse to your RO2 game path (e.g., `/path/to/SHIPPING/Rag2.exe`)
   - Click "Launch Game"

### Running All Servers

Each server runs independently:

```bash
# Terminal 1: Login Server
cargo run -p ro2-login

# Terminal 2: Lobby Server
cargo run -p ro2-lobby

# Terminal 3: World Server
cargo run -p ro2-world
```

## Database

SQLite by default, with MySQL support:

```bash
# Run with SQLite (default)
cargo run -p ro2-login

# Run with MySQL
cargo run -p ro2-login --features mysql
```

## Reverse Engineering Methodology

1. **Static Analysis** - Ghidra analysis of Rag2.exe client
2. **String Extraction** - Discovered 660+ RMI message names
3. **Structure Recovery** - Identified packet formats (16-48 bytes)
4. **Network Capture** - Wireshark analysis of real traffic (planned)
5. **Incremental Implementation** - Build and validate one message at a time

## Key Findings

### ProudNet RMI Messages
- **Req** (Request): Client → Server (201 messages)
- **Ans** (Answer): Server → Client response (201 messages)
- **Nfy** (Notify): Server → Client push (201 messages)
- **Ack** (Acknowledgment): Server → Client (57 messages)

### Packet Structures
- **PacketHeader**: 16 bytes (IP, port, host ID)
- **PacketBuffer**: 25 bytes (dynamic buffer)
- **NetworkPacket**: 44 bytes (with message type)
- **CompletePacket**: 48 bytes (full transmission unit)

### Server Ports
- **7101**: Login server (authentication)
- **7201**: Lobby server (channel/character selection)
- **7401**: World server (gameplay)

## Contributing

This is an educational project. Contributions welcome for:
- Protocol reverse engineering
- Packet structure documentation
- Server implementation
- Database schema design
- Testing with real client

## Legal Disclaimer

This project is for **educational and research purposes only**. 

- This is a clean-room implementation based on reverse engineering
- No proprietary code or assets from the official game are used
- No official server code was accessed or referenced
- All findings are from legitimate reverse engineering techniques

## License

MIT License - See [LICENSE](LICENSE) file for details

## Credits

- **Gravity Interactive** - Original Ragnarok Online 2 game
- **Nettention** - ProudNet game networking library
- **Ghidra Project** - Software reverse engineering tool
- **Rust Community** - Amazing language and ecosystem

## Resources

- [Ghidra SRE](https://ghidra-sre.org/)
- [Wireshark](https://www.wireshark.org/)
- [ProudNet Info](https://www.nettention.com/) (Korean, limited English docs)
- [Rust Networking with Tokio](https://tokio.rs/)

---

**Status:** Pre-Alpha | **Version:** 0.1.0 | **Last Updated:** January 2026
