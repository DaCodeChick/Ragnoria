# Packet Capture Guide for Ragnoria

This guide explains how to capture network traffic from the Ragnarok Online 2 client to analyze the ProudNet protocol and extract message IDs.

## Prerequisites

- Wireshark installed
- RO2 client installed at `C:\Gravity\Ragnarok Online 2 - Jawaii\SHIPPING\`
- Admin/root privileges for packet capture

## Capture Setup

### 1. Configure Wireshark

**Capture Filter** (set before starting capture):
```
tcp port 7101 or tcp port 7201 or tcp port 7401
```

**Display Filter** (use after capture):
```
tcp.port == 7101 || tcp.port == 7201 || tcp.port == 7401
```

### 2. Start Capture

1. Launch Wireshark as Administrator
2. Select your network interface (usually Ethernet or Wi-Fi)
3. Apply capture filter: `tcp port 7101 or tcp port 7201 or tcp port 7401`
4. Click "Start" to begin capturing

### 3. Capture Login Flow

**Scenario: Fresh Login**

1. Start Wireshark capture
2. Launch `Rag2.exe`
3. Enter credentials at login screen
4. Click "Login"
5. Wait for character selection screen to load
6. Stop capture

**Expected Traffic:**
```
Client -> Server (7101): TCP SYN/ACK handshake
Client -> Server (7101): ReqLogin packet
Server -> Client (7101): AnsLogin packet (with session key)
Client -> Server (7101): ReqServerStatus
Server -> Client (7101): AckServerStatus
Client -> Server (7101): TCP FIN (disconnect)

Client -> Server (7201): TCP SYN/ACK handshake (lobby connection)
Client -> Server (7201): Session validation + character list request
Server -> Client (7201): Character list
```

### 4. Export Captures

**Option A: Save as PCAPNG (recommended)**
```
File > Export Specified Packets
Format: pcapng
Save to: docs/captures/login_flow.pcapng
```

**Option B: Export as Hex Dump**
```
File > Export Packet Dissections > As Plain Text
Enable: Packet summary line, Packet details, Packet bytes (hex dump)
Save to: docs/captures/login_flow_hex.txt
```

## Analyzing Captured Packets

### Structure to Look For

Every ProudNet RMI packet should have this structure (from Ghidra analysis):

```
Offset | Size | Field Name        | Description
-------|------|-------------------|----------------------------------
0x00   | 4    | magic/signature   | ProudNet magic (unknown value)
0x04   | 4    | packet_length     | Total packet size
0x08   | 2    | message_id        | RMI message type identifier ⚠️ THIS IS WHAT WE NEED
0x0A   | 2    | flags/version     | Protocol flags or version
0x0C   | 4    | sequence_number   | Packet sequence counter
0x10   | ... | payload           | Message-specific data
```

### Step-by-Step Analysis

#### 1. Identify ReqLogin Packet

Look for the **first client → server (7101) packet with substantial data** after TCP handshake.

**In Wireshark:**
- Right-click packet → Follow → TCP Stream
- Look for first data packet after handshake
- Note the hex bytes in the bottom pane

**Example packet (hypothetical):**
```
00000000  50 52 4F 55  XX XX XX XX  01 23 00 01  00 00 00 05   PROU....#......
00000010  61 64 6D 69  6E 00 00 00  08 70 61 73  73 77 6F 72   admin....passwor
00000020  64 31 32 33                                           d123
```

**Analysis:**
- Bytes `0x08-0x09`: `01 23` → Message ID = **0x0123** (ReqLogin)
- Bytes `0x10+`: Payload containing username/password

#### 2. Identify AnsLogin Packet

Look for the **first server → client (7101) response packet**.

**Example response:**
```
00000000  50 52 4F 55  XX XX XX XX  01 24 00 01  00 00 00 00   PROU....$.......
00000010  20 AB CD EF  12 34 56 78  9A BC DE F0  11 22 33 44   ....4Vx...."3D
00000020  55 66 77 88  99 AA BB CC  DD EE FF 00               Ufw.........
```

**Analysis:**
- Bytes `0x08-0x09`: `01 24` → Message ID = **0x0124** (AnsLogin)
- Bytes `0x10+`: Session key (32 bytes in this example)

### 5. Document Findings

Create a mapping file in `docs/captures/message_id_mapping.md`:

```markdown
# Message ID Mapping (Extracted from Captures)

| Message Name | Message ID | Direction | Port | Notes |
|--------------|-----------|-----------|------|-------|
| ReqLogin | 0x0123 | C→S | 7101 | Username + password in payload |
| AnsLogin | 0x0124 | S→C | 7101 | Contains session key (32 bytes) |
| ReqServerStatus | 0x0125 | C→S | 7101 | No payload |
| AckServerStatus | 0x0126 | S→C | 7101 | Server list + population |
```

## Using the Packet Analyzer Tool

We've created a utility to parse hex dumps automatically:

```bash
# From project root
cargo run --bin packet-analyzer -- docs/captures/login_flow_hex.txt

# Or analyze a specific packet manually
cargo run --bin packet-analyzer -- --hex "50524F55..."
```

## Common Issues

### Issue: Encrypted Packets

**Symptom:** Payload looks like random bytes after packet header

**Solution:** 
1. Capture the initial connection handshake - there may be a plaintext key exchange
2. Look for RSA public key exchange in first few packets
3. Check Ghidra for AES key derivation functions (search for "CryptoPP" or "Crypto")

### Issue: No Traffic Captured

**Symptom:** Wireshark shows no packets on ports 7101/7201/7401

**Possible Causes:**
- Client configured for different server IP (check client config files)
- Using localhost/127.0.0.1 (Wireshark won't capture loopback on some systems)
- Firewall blocking capture

**Solution:**
```bash
# Windows: Use RawCap for loopback capture
RawCap.exe 127.0.0.1 loopback_capture.pcap

# Linux: Capture on 'lo' interface
tcpdump -i lo -w capture.pcap port 7101 or port 7201 or port 7401
```

### Issue: Can't Read Hex Dump

**Symptom:** Exported text file is formatted oddly

**Solution:** Use Wireshark's built-in hex dump export:
1. Select packet in packet list
2. Right-click → Copy → ...as Hex Dump
3. Paste into text editor

## Next Steps After Capture

1. **Extract Message IDs** - Update `crates/ro2-common/src/protocol/mod.rs`
2. **Map Payload Structures** - Document how username/password/session keys are encoded
3. **Identify Encryption** - Check if packets are encrypted after initial handshake
4. **Update RFC** - Add actual hex examples to `docs/protocol/RFC-RO2-PROTOCOL.md`

## Example: Complete Login Flow Analysis

See `docs/captures/EXAMPLE-ANALYSIS.md` for a complete annotated example of analyzing a full login sequence.
