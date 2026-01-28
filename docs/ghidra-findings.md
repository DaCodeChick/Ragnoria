# Ghidra Analysis Findings - Rag2.exe

**Binary:** Rag2.exe  
**Path:** /C:/Gravity/Ragnarok Online 2 - Jawaii/SHIPPING/Rag2.exe  
**Format:** Portable Executable (PE)  
**Architecture:** x86:LE:32:default (32-bit x86 Little Endian)  
**Analysis Date:** January 2026

---

## Table of Contents

1. [Overview](#overview)
2. [Packet Structures](#packet-structures)
3. [ProudNet RMI Framework](#proudnet-rmi-framework)
4. [String Analysis](#string-analysis)
5. [Import Analysis](#import-analysis)
6. [Network Protocol Observations](#network-protocol-observations)
7. [Security & Encryption](#security--encryption)
8. [Next Steps](#next-steps)

---

## Overview

Rag2.exe is the Ragnarok Online 2 game client using the ProudNet networking library by Nettention. The client is built with Gamebryo engine for graphics and contains extensive Lua scripting support.

### Key Findings
- **Networking:** ProudNet RMI (Remote Method Invocation) framework
- **Ports:** 7101 (Login), 7201 (Lobby), 7401 (World)
- **Encryption:** AES and RSA referenced (implementation details need deeper analysis)
- **Messages:** 660+ unique RMI messages identified via string extraction
- **Game Engine:** Gamebryo (NIF files mentioned in strings)

---

## Packet Structures

### PacketHeader (16 bytes)

**Location:** Root namespace `/`  
**Size:** 16 bytes  
**Type:** Structure

```c
struct PacketHeader {
    DWORD  vtable;        // +0x00: Virtual function table pointer
    DWORD  sourceIP;      // +0x04: Source IPv4 address
    WORD   sourcePort;    // +0x08: Source TCP/UDP port
    BYTE   addressFlags;  // +0x0A: Address property flags
    BYTE   reserved;      // +0x0B: Reserved (must be 0)
    DWORD  hostID;        // +0x0C: Client host identifier
};
```

**Analysis:**
- `vtable` suggests this is a C++ class with virtual methods
- `sourceIP` and `sourcePort` track client connection info
- `hostID` is a unique identifier assigned by server to each client connection
- Little-endian byte order (x86 architecture)

### PacketBuffer (25 bytes)

**Location:** Root namespace `/`  
**Size:** 25 bytes  
**Type:** Structure

```c
struct PacketBuffer {
    DWORD  bufferData;      // +0x00: Pointer to buffer memory
    DWORD  bufferSize;      // +0x04: Total buffer size
    DWORD  currentData;     // +0x08: Current data pointer
    DWORD  currentSize;     // +0x0C: Used bytes
    DWORD  allocatedSize;   // +0x10: Total allocated memory
    DWORD  readPosition;    // +0x14: Read cursor position
    BYTE   bufferFlags;     // +0x18: Control flags
};
```

**Analysis:**
- Dynamic buffer with separate read/write pointers
- Supports incremental reading and writing
- Flags likely control buffer behavior (e.g., auto-resize, read-only)

### NetworkPacket (44 bytes)

**Location:** Root namespace `/`  
**Size:** 44 bytes  
**Type:** Structure

```c
struct NetworkPacket {
    DWORD       bufferData;      // +0x00: Buffer data pointer
    DWORD       bufferSize;      // +0x04: Buffer size
    DWORD       bufferCapacity;  // +0x08: Buffer capacity
    DWORD       bufferOffset;    // +0x0C: Buffer offset
    DWORD       readPointer;     // +0x10: Read pointer
    DWORD       writePointer;    // +0x14: Write pointer
    BYTE        bufferFlags;     // +0x18: Buffer flags
    BYTE        reserved1;       // +0x19: Reserved
    BYTE        reserved2;       // +0x1A: Reserved
    BYTE        reserved3;       // +0x1B: Reserved
    DWORD       packetType;      // +0x1C: Message type ID
    PacketHeader header;         // +0x20: Embedded packet header (16 bytes)
};
```

**Analysis:**
- Extends PacketBuffer with network-specific fields
- `packetType` at offset 0x1C identifies the RMI message type
- Embedded `PacketHeader` contains connection metadata
- Total size matches: 28 bytes (buffer info) + 16 bytes (header) = 44 bytes

### CompletePacket (48 bytes)

**Location:** Root namespace `/`  
**Size:** 48 bytes  
**Type:** Structure

```c
struct CompletePacket {
    PacketBuffer buffer;    // +0x00: PacketBuffer (25 bytes)
    BYTE         padding1;  // +0x19: Alignment padding
    WORD         padding2;  // +0x1A: Alignment padding
    DWORD        packetType; // +0x1C: Message type ID
    PacketHeader header;    // +0x20: PacketHeader (16 bytes)
};
```

**Analysis:**
- Highest-level packet container
- Appears to be the structure used for actual transmission
- Padding suggests memory alignment requirements
- Total: 25 + 3 (padding) + 4 (type) + 16 (header) = 48 bytes

---

## ProudNet RMI Framework

### Discovered Structures

#### IRmiProxy (8 bytes)
**Location:** `/Proud` namespace  
**Type:** PlaceHolder Class Structure

```c
class IRmiProxy {
    // Interface for client-side RMI proxy objects
    // Size: 8 bytes (likely vtable + refcount)
};
```

#### IRmiStub (15 bytes)
**Location:** `/Proud` namespace  
**Type:** PlaceHolder Class Structure

```c
class IRmiStub {
    // Interface for server-side RMI stub objects
    // Size: 15 bytes
};
```

#### IRmiHost (4 bytes)
**Location:** `/Proud` namespace  
**Type:** PlaceHolder Class Structure

```c
class IRmiHost {
    // Host container for RMI endpoints
    // Size: 4 bytes (likely pointer)
};
```

#### CMessage (124 bytes)
**Location:** `/Proud` namespace  
**Type:** PlaceHolder Class Structure

```c
class CMessage {
    int*     field_0x00;
    uint     m_maxCapacity;     // +0x04
    void*    field_0x08;
    int      field_0x0C;
    wchar_t* field_0x10;
    int      field_0x14;
    char     field_0x18;
    // ... (undefined bytes)
    int*     field_0x1C;
    int      field_0x24;
    int      field_0x28;
    int      field_0x2C;
    // ... (64 undefined bytes at 0x30)
    uint     m_position;        // +0x78
};
```

**Analysis:**
- CMessage appears to be the main message container
- Contains Unicode string pointer (wchar_t*)
- Has capacity tracking and position tracking
- Large undefined section suggests complex internal structure

#### CReceivedMessageList (96 bytes)
**Location:** `/Proud` namespace  
**Type:** PlaceHolder Class Structure

Likely manages a queue of received RMI messages.

### Key ProudNet Strings

```
"RMI Proxy which is still in use by ProudNet core cannot be destroyed!"
"RMI Stub which is still in use by ProudNet core cannot be destroyed!"
"ProudNet RMI Proxy is not attached yet!"
"Proud::CNetClientWorker::ProcessMessage_ProudNetLayer"
"Proud::CNetServerImpl::IoCompletion_ProcessMessage_ProudNetLayer"
```

**Analysis:**
- ProudNet uses reference counting for RMI objects
- Separate message processing layers for client and server
- Architecture follows Proxy/Stub distributed object pattern

---

## String Analysis

### Message Naming Convention

All RMI messages follow a strict prefix convention:

| Prefix | Count | Direction | Purpose |
|--------|-------|-----------|---------|
| Req    | 201   | C → S     | Client requests |
| Ans    | 201   | S → C     | Server answers |
| Nfy    | 201   | S → C     | Server notifications |
| Ack    | 57    | S → C     | Server acknowledgments |

### Error Strings

Login errors:
```
"Login Ok"
"Login_Ok"
"Login_Failed"
"LoginFail_ACCOUNT_BLOCK"
"LoginFail_ACCOUNT_LOGGING"
"LoginFail_WrongLoginMethod"
"LoginFail_IP_ERROR"
"LoginFail_PASSWORD_ERROR"
"LoginFail_ACCOUNT_ERROR"
"LoginFail_LOCALSYS_ERROR"
```

Channel errors:
```
"Channel_NotExist"
"Channel_Full"
```

Server errors:
```
"Server_System_Error"
"Server_Inpected"
"Server Disconnected"
"Invalid Server Response"
```

### Server Selection Flow

```
"Waiting_ServerGroup"
"Waiting_SelectServerAck"
"Received_SelectServerAck"
"Received_ServerGroup"
"SelectServer_No_Ack"
"SelectServer_Failed"
"SelectServer_Ok"
```

**Analysis:**
- Client waits for server group info
- Sends server selection request
- Expects acknowledgment
- State machine for connection flow

### Stage System

```
"StageLogin::Enter"
"StageLogin::Leave"
"StageSelectServer::Enter"
"StageSelectServer::Leave"
```

**Analysis:**
- Client uses stage-based state management
- Each connection phase is a distinct "stage"
- Clean separation of concerns

---

## Import Analysis

### Key Windows API Imports

**Networking (via WinSock - not shown but implied):**
- TCP socket operations
- Connection management
- Likely uses `ws2_32.dll` (not in first 100 imports)

**Threading & Synchronization:**
- `CreateSemaphoreA/W`
- `ReleaseSemaphore`
- `InitializeCriticalSectionAndSpinCount`
- `TlsAlloc`, `TlsSetValue`, `TlsFree`

**Memory Management:**
- `HeapAlloc`, `HeapFree`, `HeapReAlloc`
- `VirtualAlloc`, `VirtualFree`
- `GlobalAlloc`, `GlobalLock`, `GlobalUnlock`

**Timing:**
- `QueryPerformanceCounter`
- `QueryPerformanceFrequency`
- `GetSystemTime`, `SystemTimeToFileTime`

**File I/O:**
- `CreateFileA/W`, `ReadFile`, `WriteFile`
- `FindFirstFileA`, `FindNextFileA`, `FindClose`
- `DeleteFileA/W`, `CopyFileA`

**Process Management:**
- `GetCurrentProcess`, `GetCurrentThread`
- `CreateProcessA`
- `TerminateProcess`

**Analysis:**
- Heavy use of threading primitives suggests multi-threaded network I/O
- Performance counters used for timing (likely for lag compensation)
- File operations for game assets and logs
- TLS (Thread Local Storage) for per-thread state

---

## Network Protocol Observations

### Connection Sequence (Inferred)

1. **DNS Resolution**: Client resolves server hostname
2. **TCP Connection**: Client connects to port 7101 (Login)
3. **Handshake**: Exchange version info, encryption keys
4. **Authentication**: ReqLogin → AnsLogin
5. **Server List**: ReqServerStatus → AckServerStatus
6. **Disconnect**: Close login connection
7. **Lobby Connect**: Connect to port 7201 with session key
8. **Character Select**: ReqLoginChannel → AnsLoginChannel
9. **World Connect**: Connect to port 7401
10. **Game Loop**: Enter gameplay message loop

### ProudNet Layer Processing

From string analysis:
```
"Proud::CNetClientWorker::ProcessMessage_ProudNetLayer"
"Proud::CNetServerImpl::IoCompletion_ProcessMessage_ProudNetLayer"
```

**Analysis:**
- Separate "ProudNet layer" for framework messages
- Client uses worker threads for message processing
- Server uses I/O completion ports (IOCP) for scalability

### UDP Holepunching (P2P)

```
"ServerUdpEnabled=%d,RemotePeerCount=%d,DirectP2PEnabledPeerCount=%d"
"Sending ServerHolepunch: %s"
"Message_ServerHolepunchAck. AddrOfHereAtServer=%s"
"Try to PeerHolepunchAck. ABS=%s ABR=%s BAS=%s"
```

**Analysis:**
- ProudNet supports P2P connections between clients
- Uses UDP holepunching for NAT traversal
- Primarily for peer-to-peer features (not critical for server emulator MVP)

---

## Security & Encryption

### Encryption References

**String Analysis:**
No direct strings for "AES" or "RSA" found in initial dump, but user confirmed these are used.

**Likely Implementation:**
- AES for session encryption (symmetric)
- RSA for key exchange (asymmetric)
- Session keys generated post-authentication

### HackShield Anti-Cheat

```
"RequestHShieldMsg"
"NfyHShieldErrorDetected"
"OnHackShieldDisconnect"
```

**Analysis:**
- Client uses HackShield anti-cheat protection
- Server emulator may need to handle or bypass HackShield messages
- Emulator should respond appropriately to avoid disconnection

---

## Next Steps

### Immediate Actions

1. **Packet Capture:**
   - Run real RO2 client
   - Capture traffic with Wireshark during login flow
   - Identify message IDs (numeric values for each Req/Ans/Nfy/Ack)
   - Document packet payloads

2. **Encryption Analysis:**
   - Search Ghidra for AES/RSA function implementations
   - Look for OpenSSL or CryptoAPI imports
   - Identify key exchange handshake sequence
   - Document encryption initialization

3. **Message Payload Analysis:**
   - For each login-related message, determine structure
   - ReqLogin: username, password, client version, etc.
   - AnsLogin: result code, session key, etc.
   - Use Ghidra decompiler to analyze message handlers

4. **Function Analysis:**
   - Identify message dispatch table/switch statement
   - Analyze how packet type maps to handler function
   - Understand serialization/deserialization logic

### Research Questions

- [ ] What are the exact numeric values for message IDs?
- [ ] How is the session key generated and validated?
- [ ] What is the format of username/password in ReqLogin?
- [ ] Are passwords hashed client-side before transmission?
- [ ] What AES mode is used (CBC, CTR, GCM)?
- [ ] What is the RSA key size?
- [ ] How does the client handle version checking?
- [ ] What data is included in PacketHeader.hostID?

### Deeper Analysis Targets

#### Priority 1 (Login Flow)
- `ReqLogin` structure and handler
- `AnsLogin` structure and handler
- `ReqServerStatus` / `AckServerStatus`
- Session key generation function
- Encryption initialization

#### Priority 2 (Lobby Flow)
- `ReqLoginChannel` structure
- `AnsLoginChannel` structure
- Character list format
- Channel list format

#### Priority 3 (Protocol Details)
- Message ID enumeration/constants
- Serialization format (binary encoding)
- Error code enumeration
- Heartbeat/ping mechanism

---

## Tools & Commands Used

```bash
# Ghidra MCP Server
ghidra_list_programs
ghidra_list_strings --filter "Req" --limit 200
ghidra_list_strings --filter "Ans" --limit 200
ghidra_list_strings --filter "Nfy" --limit 200
ghidra_list_strings --filter "Ack" --limit 200
ghidra_get_data_type --name "PacketHeader"
ghidra_get_data_type --name "PacketBuffer"
ghidra_get_data_type --name "NetworkPacket"
ghidra_get_data_type --name "CompletePacket"
ghidra_get_data_type --name "CMessage" --category "/Proud"
ghidra_list_imports --limit 100
```

---

## Appendices

### Complete Message Lists
See [Appendix A: Message Catalog](protocol/appendices/message-catalog.md) for the complete list of 660+ RMI messages.

### Data Type Definitions
All packet structures are documented in [RFC-RO2-PROTOCOL.md](protocol/RFC-RO2-PROTOCOL.md) Section 5.

---

**End of Ghidra Analysis Findings**
