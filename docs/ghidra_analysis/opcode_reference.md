# Complete Opcode Reference - Ragnarok Online 2

Comprehensive mapping of all network opcodes identified through Ghidra analysis and PCAP inspection.

---

## Opcode Ranges

| Range | Layer | Direction | Purpose |
|-------|-------|-----------|---------|
| 0x01-0x32 | ProudNet Protocol | Both | Low-level networking (encryption, compression, P2P) |
| 0x2EE0-0x2EFF | Game - Login/Auth | Client → Server | Authentication requests |
| 0x30D0-0x30DF | Game - Login/Auth | Server → Client | Authentication responses |
| 0x1000+ | Game - Various | Both | Game logic, character, chat, world, etc. |

---

## ProudNet Protocol Layer (0x01-0x32)

These opcodes handle the underlying network protocol. All game messages are wrapped inside this layer.

| Opcode | Name | Direction | Purpose | Validation Required |
|--------|------|-----------|---------|---------------------|
| 0x01 | KeepAlive | Both | Connection maintenance ping | No |
| 0x02 | Ping | Client → Server | Latency measurement request | No |
| 0x03 | Timeout | Server → Client | Connection timeout notification | Yes |
| 0x04 | EncryptionHandshake | Server → Client | RSA public key exchange | Yes |
| 0x05 | EncryptionResponse | Client → Server | AES session key (RSA encrypted) | No |
| 0x06 | ServerInfo | Server → Client | Server information exchange | Yes |
| 0x07 | VersionCheck | Client → Server | ProudNet protocol version | No |
| 0x08 | PingResponse | Server → Client | Ping reply with timestamp | Yes |
| 0x09 | EchoResponse | Server → Client | Echo test response | Yes |
| 0x0A | ConnectionSuccess | Server → Client | Connection establishment confirmation | Yes |
| 0x0B | VersionCheckAck | Server → Client | Protocol version validation result | Yes |
| 0x0D | ServerHolepunchAck | Server → Client | NAT hole punching acknowledgment | Yes |
| 0x0F | ReliableUDP | Both | UDP packet reliability layer | Yes |
| 0x11 | P2PGroup | Both | Peer-to-peer group management | Yes |
| 0x13 | ReliablePacket | Both | Reliable transmission with retransmit | No |
| 0x18 | HeartbeatAck | Server → Client | Heartbeat acknowledgment | Yes |
| 0x19 | DirectP2P | Both | Direct peer-to-peer communication | Yes |
| 0x1A | IndirectP2P | Both | Server-mediated P2P communication | Yes |
| 0x1B | Heartbeat | Client → Server | Heartbeat with timestamp | No |
| 0x1C | KeepAlivePing | Client → Server | Fast connection check | No |
| 0x1D | P2PConnectionRequest | Both | P2P connection initiation | Yes |
| 0x1E | Reserved | - | Unused/reserved | - |
| 0x1F | ConnectTCP | Both | TCP connection establishment | No |
| 0x20 | DisconnectTCP | Both | TCP disconnection | No |
| 0x21 | UDPHolePunch | Both | UDP NAT traversal | No |
| 0x22 | P2PJoinRequest | Both | Join P2P group request | No |
| 0x23 | P2PRelayData | Both | Server-relayed P2P data | Yes |
| 0x24 | SpeedHack | Client → Server | Anti-cheat speed detection | No |
| 0x25 | EncryptedPacket | Both | AES encrypted payload (variant 1) | Recursive |
| 0x26 | EncryptedPacket | Both | AES encrypted payload (variant 2) | Recursive |
| 0x27 | CompressedPacket | Both | Compressed payload | Recursive |
| 0x28 | FrameworkVersion | Both | ProudNet framework version negotiation | No |
| 0x29 | P2PStatus | Both | Peer-to-peer status reporting | No |
| 0x2F | PolicyRequest | Client → Server | XML policy file request | No |
| 0x30 | Reserved | - | Unused/reserved | - |
| 0x31 | RequestConnectToClient | Server → Client | Request client connection | No |
| 0x32 | ClientConnectionAck | Client → Server | Client connection acknowledgment | No |

### Notes on ProudNet Layer:
- **Recursive Processing**: Opcodes 0x25, 0x26, 0x27 decrypt/decompress and then recursively dispatch
- **Validation**: Most opcodes call `ValidateProudNetPacketContext()` before processing
- **Encryption**: All game messages SHOULD be wrapped in 0x25 or 0x26 for security

---

## Game Layer - Authentication (Client → Server)

Client-initiated login and authentication requests.

| Opcode | Name | Size | Purpose | Handler Offset |
|--------|------|------|---------|----------------|
| 0x2EE1 | ReqVersionCheck | Variable | Client version validation | +0x24 |
| 0x2EE2 | **ReqLogin** | **211 bytes** | **Primary login authentication** | **+0x20** |
| 0x2EE3 | ReqGraLogin | Variable | Gravity-specific login variant | +0x24 |
| 0x2EE4 | ReqChannelList | Minimal | Request server channel list | +0x24 |
| 0x2EE5 | ReqLoginChannel | ~8 bytes | Connect to specific channel | +0x2C |
| 0x2EE6 | ReqLogOut | Minimal | Logout from authentication server | +0x30 |
| 0x2EE7 | SendPacket | Variable | Generic packet forwarding | +0x34 |
| 0x2EE8 | ReqServerStatus | Minimal | Request server/channel status | +0x38 |
| 0x2EE9 | ReqSecondPassword | Minimal | Request second password prompt | +0x3C |
| 0x2EEA | ReqInputSecondPassword | Variable | Submit second password (2FA) | +0x40 |
| 0x2EEC | ReqSteamLogin | Variable | Steam platform authentication | Unknown |
| 0x2EED | ReqAeriaGamesLogin | Variable | AeriaGames platform authentication | Unknown |

### 0x2EE2: ReqLogin (CRITICAL)

**Total Size**: 211 bytes (2-byte opcode + 209-byte payload)

**Payload Structure** (209 bytes / 0xD1):
```
Unknown exact field layout, but contains:
- Account username (likely null-terminated string, max ~32 chars)
- Password (hashed or encrypted, likely 16-32 bytes)
- Client version information (build number, etc.)
- Platform identifier (Steam, standard, etc.)
- Session tokens or hardware fingerprint
- Possibly padding to reach 209 bytes
```

**Serialization**: Raw binary dump via `SerializeReqLoginPacket()` - simple memcpy of structure

**Security**: MUST be encrypted in 0x25/0x26 wrapper before transmission

**Builder Function**: `SendReqLogin` @ 0x00E52FE0

---

## Game Layer - Authentication (Server → Client)

Server responses to authentication requests.

| Opcode | Name | Size | Purpose | Handler Offset |
|--------|------|------|---------|----------------|
| 0x30D4 | AckVersionCheck | Variable | Version validation result | +0x24 |
| 0x30D5 | **AckLogin** | **82 bytes** | **Login authentication result** | **+0x24** |
| 0x30D7 | AckChannelList | Variable | Available server channel list | +0x24 |
| 0x30D8 | AckLoginChannel | Variable | Channel connection result | +0x28 |
| 0x30D9 | SendPacketAck | Variable | Generic packet acknowledgment | +0x2C |
| 0x30DA | AnsSecondPassword | Variable | Second password prompt response | +0x30 |
| 0x30DB | AnsInputSecondPassword | Variable | Second password validation result | +0x34 |

### 0x30D5: AckLogin (CRITICAL)

**Total Size**: 82 bytes (2-byte opcode + 80-byte payload)

**Payload Structure** (80 bytes / 0x50):
```
Unknown exact field layout, but contains:
- Result code (success/failure enum, likely 4 bytes)
- Account ID (likely uint32 or uint64)
- Session identifier/token (16-32 bytes)
- Character server connection info (IP, port, token)
- Account status flags (banned, premium, GM, etc.)
- Possibly error message ID for failed logins
```

**Deserialization**: `DeserializeAckLogin()` - reads exactly 80 bytes

**Possible Result Codes** (from string analysis):
```c
enum LoginResult {
    Login_Ok,                      // Successful authentication
    Login_Failed,                  // Generic failure
    LoginFail_ACCOUNT_BLOCK,       // Account banned/suspended
    LoginFail_ACCOUNT_LOGGING,     // Account already logged in
    LoginFail_WrongLoginMethod,    // Invalid authentication method
    LoginFail_IP_ERROR,            // IP restriction violation
    LoginFail_PASSWORD_ERROR,      // Invalid password
    LoginFail_ACCOUNT_ERROR,       // Account doesn't exist
    LoginFail_LOCALSYS_ERROR,      // Server-side error
    CannotFind_Account,            // Account lookup failed
};
```

---

## Game Layer - Mystery Handshake

### 0x0000: InitialHandshake (UNCONFIRMED NAME)

**Direction**: Both (Client → Server, Server → Client)

**Size**: 26 bytes (decrypted payload, excluding 2-byte opcode)

**Total Packet Size**: ~40 bytes (including ProudNet framing and encryption overhead)

**Purpose**: Unknown game-layer handshake that occurs after ProudNet encryption is established but before login credentials are sent

**Structure**:
```
Offset | Size | Type    | Field Name           | Client Value       | Server Value
-------|------|---------|----------------------|--------------------|------------------
0x00   | 2    | uint16  | Opcode               | 0x0000             | 0x0000
0x02   | 2    | uint16  | Version              | 0x01E1 (481)       | MIRROR CLIENT
0x04   | 2    | uint16  | Build                | 0x2E10 (4142)      | MIRROR CLIENT
0x06   | 2    | uint16  | Unknown_Field1       | 0x0021 (33)        | MIRROR CLIENT
0x08   | 4    | uint32  | GUID/Timestamp       | 0xCBA416F1         | DIFFERENT (server timestamp)
0x0C   | 2    | uint16  | Unknown_Field2       | 0x0001 (1)         | MIRROR CLIENT
0x0E   | 4    | uint32  | Status/Flags         | 0x00000001 (1)     | MIRROR CLIENT (CRITICAL!)
0x12   | 4    | uint32  | Unknown_Field3       | 0x07022500         | MIRROR CLIENT
0x16   | 4    | uint32  | Mysterious_Field     | 0x803F0000         | MIRROR CLIENT (SUSPECTED CAUSE!)
-------|------|---------|----------------------|--------------------|------------------
Total: 26 bytes
```

**Critical Findings**:
- **Bytes 0x0E-0x11 (Status)**: MUST be 0x00000001 (not 0x00000000)
  - We fixed this bug earlier
  - Setting to 0x00 causes immediate connection issues
  
- **Bytes 0x16-0x19 (0x803F0000)**: HIGHLY SUSPICIOUS
  - Appears 20 times in binary at various addresses
  - Could be: version constant, capability flag, magic number, or two 16-bit values
  - If interpreted as big-endian float: 0x3F800000 = 1.0
  - If two uint16: 0x3F80 (16256) + 0x0000 (0)
  - **HYPOTHESIS**: This field is validated and our mirroring might be wrong

**Handler**: NOT FOUND in Ghidra
- Expected location: Game message dispatcher
- Likely explanation: Inlined, obfuscated, or called via COM interface

**PCAP Evidence**:
```
Frame 18: Client sends 0x0000 (40 bytes encrypted)
Frame 20: Server sends 0x0000 response (40 bytes encrypted)
Frame 22: Client sends 0x2EE2 login packet (234 bytes encrypted)
Time delta: 14.7 milliseconds (nearly instant)
```

**Implications**:
- Login proceeds immediately after receiving 0x0000 in official server PCAP
- Our server sends 0x0000, but client doesn't enable login button
- **Validation check is failing on our response**

---

## Byte Sequence Analysis

### 0x803F0000 - The Smoking Gun?

**Found At** (20 occurrences):
```
Address    | Context
-----------|----------------------------------------------------------
0x00cd1c8c | In function FUN_00cd1be0
0x01357cee | Data section (possible constant table)
0x0140a94a | Data section
0x0159a23e | Data section (examined, looks like float constants)
0x015a54ae | Data section (multiple occurrences nearby)
... | 15 more occurrences in data sections
```

**Hexdump at 0x0159a23e**:
```
159a23e  80 3f 00 00 80 3f 00 00  80 3f 88 88 35 01 7c 88  |.?...?...?..5.|.|
159a24e  35 01 6c 88 35 01 60 88  35 01 50 88 35 01 3c 88  |5.l.5.`.5.P.5.<.|
```

**Interpretation Possibilities**:

1. **Big-Endian Float 1.0**: 0x3F800000 = 1.0
   - Makes sense for a constant table
   - But why would client/server exchange float 1.0?
   - **Hypothesis**: Protocol version as floating point?

2. **Two 16-bit Values**: 0x3F80 + 0x0000
   - 0x3F80 = 16256 decimal
   - Could be: protocol version 16256, flags 0
   - **Hypothesis**: ProudNet version + feature flags?

3. **Magic Constant**: Required validation value
   - Client expects specific value
   - Server must respond with EXACT match
   - **Hypothesis**: Anti-tampering check?

4. **Capability Flags**: Bit fields
   - Bits indicate supported features
   - Server must advertise correct capabilities
   - **Hypothesis**: 0x803F = feature set, 0x0000 = reserved?

**Test Strategy**:
```rust
// Try different interpretations
test_values = [
    0x803F0000,  // Original (current)
    0x00003F80,  // Byte-swapped 16-bit
    0x3F800000,  // Float 1.0 big-endian
    0x0000803F,  // Float 1.0 little-endian
    0x00000000,  // All zeros
    0xFFFFFFFF,  // All ones
    0x3F803F80,  // Duplicate 16-bit value
];
```

---

## Platform-Specific Authentication

Different publishers use different opcodes and structures.

| Platform | Request Opcode | Notes |
|----------|----------------|-------|
| Standard | 0x2EE2 | Username + password |
| Gravity | 0x2EE3 | Gravity-specific variant |
| Steam | 0x2EEC | Steam ticket validation |
| AeriaGames | 0x2EED | AeriaGames SSO token |
| GameForge | Unknown | Likely custom opcode |
| Lyto | Unknown | Likely custom opcode |
| DreamSquare | Unknown | Likely custom opcode |

All variants use similar payload structure but with platform-specific authentication tokens replacing username/password.

---

## Encryption Wrapper Format

All sensitive game messages are wrapped in ProudNet encryption:

```
┌────────────────────────────────────────────────────────────────┐
│ ProudNet Frame Header (variable size)                          │
├────────────────────────────────────────────────────────────────┤
│ Opcode: 0x25 or 0x26 (2 bytes)                                 │
├────────────────────────────────────────────────────────────────┤
│ Encrypted Payload (AES-128 CBC):                               │
│   ┌────────────────────────────────────────────────────────────┤
│   │ Inner Opcode (2 bytes): 0x0000, 0x2EE2, etc.               │
│   ├────────────────────────────────────────────────────────────┤
│   │ Inner Payload (variable size)                              │
│   └────────────────────────────────────────────────────────────┤
├────────────────────────────────────────────────────────────────┤
│ ProudNet Frame Footer (checksums, etc.)                        │
└────────────────────────────────────────────────────────────────┘
```

**Encryption Details**:
- Algorithm: AES-128, possibly AES-192 or AES-256
- Mode: CBC (Cipher Block Chaining)
- IV: Derived from session key exchange (0x04 handshake)
- Key: Established via RSA-1024 encrypted exchange (0x05)

---

## Summary Statistics

- **Total Opcodes Identified**: 60+
- **ProudNet Protocol**: 30 opcodes (0x01-0x32)
- **Authentication Requests**: 11 opcodes (0x2EE1-0x2EED)
- **Authentication Responses**: 7 opcodes (0x30D4-0x30DB)
- **Critical Packets**: 3 (0x0000, 0x2EE2, 0x30D5)
- **Unsolved Mystery**: 1 (0x0000 validation logic)

---

## Dispatcher Function Summary

| Function | Address | Opcodes Handled | Purpose |
|----------|---------|-----------------|---------|
| DispatchProudNetProtocolPackets | 0x00F43FF0 | 0x01-0x32 | ProudNet layer |
| DispatchLoginAuthPackets | 0x00E552E0 | 0x2EE1-0x2EED | Client auth requests |
| DispatchAckPackets_0x30D5_0x30DC | 0x00E58940 | 0x30D4-0x30DC | Server auth responses |
| [Unknown 0x0000 Handler] | ??? | 0x0000 | Mystery handshake |

---

## Next Steps

1. **Capture Official Server**: Get PCAP with successful 0x0000 → 0x2EE2 sequence
2. **Compare Bytes**: Find differences in 0x0000 response
3. **Test Field Variations**: Systematically try different values for bytes 0x16-0x19
4. **Dynamic Analysis**: Use x64dbg to trace 0x0000 handler execution

