# RO2 Login Server - Reverse Engineering Findings

## Date: 2026-01-31

## Executive Summary

Investigation into why the RO2 client displays the login UI but fails to send the 0x2EE2 (ReqLogin) packet when the Login button is clicked. The client successfully completes ProudNet handshake and receives our 0x0000 response, but some validation check is preventing login from proceeding.

---

## Client Binary Analysis (Rag2.exe)

### Key Functions Identified

#### 1. Login Packet Dispatcher: `DispatchLoginAuthPackets` (0x00E552E0)
- **Size**: 13,765 bytes (0x35C5)
- **Purpose**: Routes incoming login-related packets based on opcode
- **Architecture**: Large switch statement with virtual function delegate calls
- **Handles**: Opcodes 0x2EE1 - 0x2EED (client requests)

**Key Cases**:
```c
case 0x2EE1: ReqVersionCheck (version validation)
case 0x2EE2: ReqLogin (primary authentication - 211 bytes total, 209 payload)
case 0x2EE3: ReqGraLogin (Gravity-specific login)
case 0x2EE4: ReqChannelList (request server list)
case 0x2EE5: ReqLoginChannel (connect to channel)
case 0x2EE6: ReqLogOut (disconnect from auth server)
case 0x2EE8: ReqServerStatus (server status query)
case 0x2EE9: ReqSecondPassword (2FA request)
case 0x2EEA: ReqInputSecondPassword (2FA input)
case 0x2EEC: ReqSteamLogin (Steam authentication)
case 0x2EED: ReqAeriaGamesLogin (AeriaGames authentication)
```

#### 2. Acknowledgment Dispatcher: `DispatchAckPackets_0x30D5_0x30DC` (0x00E58940)
- **Purpose**: Routes server response packets
- **Handles**: Opcodes 0x30D4 - 0x30DC (server responses)

**Key Cases**:
```c
case 0x30D4: AckVersionCheck (version validation result)
case 0x30D5: AckLogin (login result - 82 bytes total, 80 payload)
case 0x30D7: AckChannelList (channel list data)
case 0x30D8: AckLoginChannel (channel connection result)
case 0x30DA: AnsSecondPassword (2FA response)
case 0x30DB: AnsInputSecondPassword (2FA validation result)
```

#### 3. ProudNet Protocol Dispatcher: `DispatchProudNetProtocolPackets` (0x00F43FF0)
- **Purpose**: Handles ProudNet layer protocol (opcodes 0x01-0x32)
- **Key Feature**: Recursive processing for encrypted (0x25/0x26) and compressed (0x27) packets

**Notable Protocol Opcodes**:
```c
0x01: KeepAlive
0x02: Ping
0x04: EncryptionHandshake (AES key exchange - CRITICAL)
0x07: VersionCheck
0x0A: ConnectionSuccess
0x1B: Heartbeat
0x1C: KeepAlive ping
0x25/0x26: EncryptedPacket (AES-encrypted game messages)
0x27: CompressedPacket
```

#### 4. Login Request Builder: `SendReqLogin` (0x00E52FE0)
- **Size**: 173 bytes
- **Purpose**: Constructs and sends 0x2EE2 login packet
- **Payload**: 209 bytes (0xD1) containing username, password, client version
- **Critical Finding**: **NO CALLERS FOUND** - This function is invoked via function pointer or virtual function, making it impossible to trace back to the Login button UI handler

#### 5. Deserialization Functions
```c
DeserializeReqLogin @ 0x00E5A430        (client → server)
DeserializeAckLogin @ 0x00E5A530        (server → client)
DeserializeAckVersionCheck @ 0x00E5A500 (server → client)
DeserializeReqVersionCheck @ 0x00E5A400 (client → server)
```

---

## Packet Structure Analysis

### The Mysterious 0x0000 Packet

**CLIENT SENDS** (26 bytes decrypted):
```
Offset | Bytes                    | Description
-------|--------------------------|------------------------------------------
0x00   | 00 00                    | Opcode 0x0000
0x02   | 01 E1                    | Version/Build (481 decimal)
0x04   | 2E 10                    | Build number (4142 decimal)
0x06   | 00 21                    | Unknown field
0x08   | CB A4 16 F1              | Client GUID/Timestamp
0x0C   | 00 01                    | Unknown field
0x0E   | 00 00 00 01              | Status field (CRITICAL - must be 0x00000001)
0x12   | 07 02 25 00              | Unknown field (possible version/checksum)
0x16   | 80 3F 00 00              | Mysterious field (NOT float 1.0!)
```

**SERVER RESPONDS** (26 bytes decrypted):
```
Same structure as client, except:
- Bytes 0x08-0x0B: Different GUID (server timestamp)
- All other bytes: MIRROR CLIENT EXACTLY
```

**Note on 0x803F0000 (bytes 22-25)**:
- Ghidra search found this byte sequence appears **20 times** in the binary
- Locations include: data sections, possible float constants, lookup tables
- If interpreted as float (big-endian 0x3F800000), it equals 1.0
- If interpreted as two 16-bit values: 0x3F80 + 0x0000
- **Hypothesis**: Could be a version number, capability flag, or magic constant

---

## Critical Unknowns

### 1. The 0x0000 Handler Mystery
- **Problem**: Cannot locate the client-side function that processes incoming 0x0000 responses
- **Expected Location**: Should be in game message dispatcher (not ProudNet layer)
- **Likely Explanation**: 
  - It's a custom game-layer handshake, not a standard ProudNet or login protocol message
  - Handler might be inlined, optimized away, or called via complex function pointer
  - Might not be called "0x0000" internally - could be "InitialHandshake", "GameAuth", etc.

### 2. The SendReqLogin Gating Condition
- **Problem**: No way to trace what enables the Login button to send 0x2EE2
- **Observations**:
  - `SendReqLogin` has no direct callers (function pointer invocation)
  - UI event handlers are likely COM/DirectX callbacks
  - Some client-side flag must be set to TRUE before login proceeds
- **Possible Gates**:
  - Connection state enum (e.g., `STATE_HANDSHAKE_COMPLETE`)
  - Boolean flag (e.g., `m_bAuthServerReady`)
  - Validation result from parsing 0x0000 response
  - Timeout timer or keepalive check

### 3. Validation Logic for 0x0000 Response
Cannot determine which fields are validated and what values are acceptable:
- **Bytes 14-17 (status)**: Known to require 0x00000001 (not 0x00000000)
- **Bytes 18-21 (0x07022500)**: Purpose unknown, might be validated
- **Bytes 22-25 (0x803F0000)**: Most suspicious field, appears throughout binary
- **Checksum/CRC**: Possible hidden validation we're not aware of

---

## PCAP Analysis Comparison

### Official Server Behavior
**From /home/admin/Downloads/ro2game2.pcapng**:

```
Frame 18: CLIENT → SERVER: 0x0000 (40 bytes total, 26 decrypted + 14 framing/encryption)
Frame 20: SERVER → CLIENT: 0x0000 response (40 bytes total)
Frame 22: CLIENT → SERVER: 0x2EE2 login packet (234 bytes)
         Time delta: 14.7 milliseconds
```

**Key Observation**: Login happens **IMMEDIATELY** (14.7ms) after receiving 0x0000 response. This suggests:
1. Either auto-filled credentials (no user interaction)
2. OR our 0x0000 response validation PASSES and client is ready to send pre-filled data
3. The validation is VERY fast (microseconds), so it's a simple byte check, not complex crypto

---

## Byte Sequence Investigation

### Search Results for Suspicious Bytes

**0x803F0000 appears at**:
```
00cd1c8c (in function FUN_00cd1be0)
01357cee (data section)
0140a94a (data section)
0159a23e (data section) ← Examined, appears to be float constant table
015a54ae-015a5526 (multiple occurrences in data)
```

**0x07022500**: No matches found
- Likely client-specific value (version, capability flags, etc.)
- Safe to mirror from client

---

## Security Architecture

### Encryption Flow
1. **ProudNet Handshake** (0x04): RSA-1024 key exchange
2. **AES Encryption Established**: Client and server agree on symmetric key
3. **Game Messages Wrapped**: All sensitive packets encrypted in 0x25/0x26 envelopes
4. **Recursive Decryption**: ProudNet dispatcher decrypts → dispatches to game dispatcher

### Authentication Flow (Expected)
```
1. TCP Connect → ProudNet Handshake (0x04 → 0x05 → 0x06)
2. Version Check (0x07 → 0x0A)
3. ??? Initial Handshake (0x0000 → 0x0000) ← WE ARE HERE
4. User clicks Login button
5. ReqLogin (0x2EE2 encrypted in 0x25/0x26)
6. Server validates credentials
7. AckLogin (0x30D5 encrypted)
8. [Optional] Second password (0x2EEA → 0x30DB)
9. Channel selection (0x2EE5 → 0x30D8)
10. Transition to game server
```

**PROBLEM**: Step 3 (0x0000 handshake) appears to be failing validation on client side.

---

## Error Handling (from strings analysis)

### Login Failure Codes
```c
"Login_Ok"                      // Success
"Login_Failed"                  // Generic failure
"LoginFail_ACCOUNT_BLOCK"       // Account banned
"LoginFail_ACCOUNT_LOGGING"     // Already logged in
"LoginFail_WrongLoginMethod"    // Invalid auth method
"LoginFail_IP_ERROR"            // IP restriction
"LoginFail_PASSWORD_ERROR"      // Wrong password
"LoginFail_ACCOUNT_ERROR"       // Account doesn't exist
"LoginFail_LOCALSYS_ERROR"      // Server error
"CannotFind_Account"            // Lookup failed
```

---

## Platform-Specific Authentication

The client supports multiple publisher platforms:
- **Standard**: Username + password
- **Steam** (0x2EEC): Steam ticket validation
- **AeriaGames** (0x2EED): AeriaGames SSO
- **GameForge**: GameForge account system
- **Lyto**: Lyto Games platform
- **DreamSquare**: DreamSquare platform

Each variant uses similar packet structure but with platform tokens.

---

## Current Server Implementation

### What We Send (0x0000 response)
```rust
// 26 bytes, mirroring client except GUID
[
    0x00, 0x00,                          // Opcode
    client_version[0..2],                // Mirror version
    client_build[0..2],                  // Mirror build
    client_field1[0..2],                 // Mirror field (0x0021)
    server_guid[0..4],                   // OUR TIMESTAMP (ONLY DIFFERENCE)
    client_field2[0..2],                 // Mirror field (0x0001)
    client_status[0..4],                 // Mirror status (0x00000001)
    client_field3[0..4],                 // Mirror field (0x07022500)
    client_field4[0..4],                 // Mirror field (0x803F0000) ← SUSPICIOUS
]
```

### What Happens on Client Side
```
1. Client receives encrypted 0x25/0x26 packet ✓
2. ProudNet layer decrypts payload ✓
3. Game message dispatcher sees 0x0000 opcode ✓
4. ??? Handler processes 26 bytes ???
5. ??? Validation check FAILS ???
6. ??? Connection state NOT set to "ready" ???
7. User types username/password ✓
8. User clicks Login button ✓
9. ??? SendReqLogin() NOT called ??? ← FAILURE
10. After ~10 seconds, client times out and disconnects ✓
```

---

## Hypotheses for Failure

### Hypothesis 1: Bytes 22-25 Validation (0x803F0000)
**Likelihood**: HIGH

The byte sequence 0x803F0000 appears 20 times in the binary. This could be:
- A version constant that must match exactly
- A capability flag indicating supported features
- A magic number for protocol validation
- Two 16-bit values: protocol version (0x3F80) + flags (0x0000)

**Test**: Try different values like 0x0000803F, 0x3F800000, 0x00000000, etc.

### Hypothesis 2: Bytes 18-21 Validation (0x07022500)
**Likelihood**: MEDIUM

This field could be:
- A checksum or CRC of previous fields
- A version identifier
- Capability flags

**Test**: Try all zeros, all FFs, or compute a checksum.

### Hypothesis 3: Missing Packet
**Likelihood**: MEDIUM

Maybe 0x0000 isn't supposed to exist at all? Perhaps we should be sending:
- AckVersionCheck (0x30D4) instead
- Some other initialization packet
- Nothing (client auto-proceeds after ProudNet handshake)

**Test**: Don't send 0x0000, see if client proceeds anyway.

### Hypothesis 4: Timing Issue
**Likelihood**: LOW

The PCAP shows 14.7ms delay. Maybe client expects:
- Immediate response (< 1ms)
- Specific delay window
- Multiple roundtrips before enabling login

**Test**: Add artificial delays, send multiple 0x0000 responses.

### Hypothesis 5: Hidden State Machine
**Likelihood**: MEDIUM

Client might require a specific sequence:
- ProudNet handshake → Version check → 0x0000 → Some other packet → Ready
- We might be missing a step

**Test**: Capture official server PCAP with full session from start to successful login.

---

## Recommended Next Steps

### Option 1: Official Server Testing (PRIORITY)
**Goal**: Capture real login session with valid credentials

**Steps**:
1. User provides credentials for official server
2. Modify client config to connect to official server
3. Capture full PCAP from TCP connect → successful login
4. Decrypt all packets using captured RSA key
5. Compare byte-for-byte with our implementation
6. Identify missing/incorrect fields

**Risk**: Credentials exposed in PCAP (recommend throwaway test account)

### Option 2: Dynamic Analysis (x64dbg)
**Goal**: Find the validation logic by debugging live

**Steps**:
1. Launch Rag2.exe in x64dbg
2. Set breakpoint on `SendReqLogin` (0x00E52FE0)
3. Connect to our server, click Login button
4. If breakpoint NOT hit → work backwards:
   - Find Login button click handler
   - Find condition that gates SendReqLogin call
   - Identify which flag is FALSE
5. If breakpoint IS hit → unexpected, login IS being sent (different problem)

### Option 3: Systematic Field Fuzzing
**Goal**: Brute-force find the correct field values

**Implementation**:
```rust
// Test bytes 22-25 (0x803F0000)
for val in interesting_values {
    response[22..26] = val;
    send_and_test();
}

// Test bytes 18-21 (0x07022500)
for val in interesting_values {
    response[18..22] = val;
    send_and_test();
}

// Interesting values:
// - All zeros: 0x00000000
// - All ones: 0xFFFFFFFF
// - Byte-swapped: 0x00003F80
// - Float 1.0 LE: 0x0000803F
// - Float 1.0 BE: 0x3F800000
// - Current: 0x803F0000
```

### Option 4: ProudNet Documentation Research
**Goal**: Find official documentation or SDK

Maybe ProudNet (the middleware) has public documentation that explains the handshake protocol. The 0x0000 packet might be a standard ProudNet feature, not RO2-specific.

---

## Technical Debt & Assumptions

### Assumptions We're Making
1. ✓ ProudNet encryption is implemented correctly (proven by successful decryption)
2. ✓ Packet framing is correct (proven by successful ProudNet handshake)
3. ✓ 0x0000 is a real game-layer message (seen in PCAP)
4. ✓ Client receives and processes our 0x0000 response (no errors in logs)
5. ? Our 0x0000 response structure is correct (UNVERIFIED)
6. ? Mirroring all bytes is the correct approach (UNVERIFIED)
7. ? No additional packets are needed between 0x0000 and login (UNVERIFIED)

### Known Issues
- Cannot find 0x0000 handler in Ghidra (might be obfuscated, inlined, or misidentified)
- Cannot trace Login button → SendReqLogin call chain (UI code complexity)
- No access to official server for comparison testing (until now)
- PCAP analysis incomplete (need full session with decryption keys)

---

## Conclusion

The RO2 client successfully:
1. ✓ Connects via TCP
2. ✓ Completes ProudNet handshake (RSA + AES)
3. ✓ Receives our 0x0000 response
4. ✓ Displays login UI
5. ✓ Accepts user input

But FAILS at:
6. ✗ Sending 0x2EE2 when Login button is clicked

**Root Cause**: Unknown validation check in 0x0000 response handler is failing, preventing client from entering "ready to authenticate" state.

**Most Likely Culprit**: Bytes 22-25 (0x803F0000) contain a version number, protocol flag, or magic constant that we're mirroring incorrectly.

**Best Path Forward**: Test with official server using valid credentials to capture a known-good session for comparison.

---

## Files Referenced
- Client: `/C:/Gravity/Ragnarok Online 2 - Jawaii/SHIPPING/Rag2.exe`
- PCAP: `/home/admin/Downloads/ro2game2.pcapng`
- Server: `/home/admin/Documents/GitHub/Ragnoria/crates/ro2-login/src/main.rs`
- Crypto: `/home/admin/Documents/GitHub/Ragnoria/crates/ro2-common/src/crypto.rs`

---

## Author
OpenCode AI Agent with Ghidra Analysis
Date: January 31, 2026
