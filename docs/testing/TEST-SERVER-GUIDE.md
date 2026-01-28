# Testing ProudNet Protocol Server with RO2 Client

## Overview

The test server (`test_server.rs`) is designed to test the ProudNet protocol implementation with an actual RO2 client. It will:

1. Accept client connections on port 7101
2. Perform the encryption handshake
3. Decrypt 0x25/0x26 encrypted packets in real-time
4. Extract and display game message opcodes

## Setup

### 1. Start the Test Server

```bash
cd /home/admin/Documents/GitHub/Ragnoria
cargo run --bin test_server
```

You should see:
```
==============================================
   RO2 ProudNet Protocol Test Server
==============================================

Server listening on: 0.0.0.0:7101
Waiting for connections...
==============================================
```

### 2. Configure RO2 Client

You need to point the RO2 client to your test server instead of the real server.

#### Option A: Modify Hosts File (Easiest)

**Windows:**
1. Open `C:\Windows\System32\drivers\etc\hosts` as Administrator
2. Add this line:
   ```
   127.0.0.1    loginserver.ro2.gravity.com
   ```
3. Save the file
4. The client will now connect to localhost:7101

**Linux:**
1. Open `/etc/hosts` as root
2. Add the same line
3. Save the file

#### Option B: Hex Edit Client Binary (Advanced)

If you know the server address in the binary:
1. Open `Rag2.exe` in a hex editor
2. Search for the server hostname/IP
3. Replace with `127.0.0.1` or your test server IP
4. Save as `Rag2_test.exe`

### 3. Start Wireshark (Optional)

To capture the traffic for analysis:

```bash
# Capture on loopback interface, filter for port 7101
wireshark -i lo -f "tcp port 7101" &

# Or use tshark
tshark -i lo -f "tcp port 7101" -w /tmp/ro2_test.pcapng
```

### 4. Launch RO2 Client

Start the RO2 client normally. It should connect to your test server on port 7101.

## What to Expect

### Successful Connection Flow

```
[CONNECT] New client: 127.0.0.1:xxxxx

[0x2F] Flash policy request detected
[0x2F] Sending XML policy (110 bytes, NO framing)

[0x04] Sending encryption handshake
[HEXDUMP] 0x04 packet (185 bytes):
  0000  13 57 02 b7 00 04 00 00  00 00 01 00 00 00 01 00  |.W..............|
  ...

[PACKET] Opcode: 0x05, Size: 214 bytes
[0x05] Encryption response - decrypting AES session key
[SUCCESS] Decrypted AES session key: 16 bytes
[0x06] Sending encryption ready acknowledgment

[PACKET] Opcode: 0x07, Size: 23 bytes
[0x07] Version check
[0x0A] Sending connection success (session ID: 3849572910)

[PACKET] Opcode: 0x1B, Size: 13 bytes
[0x1B] Heartbeat request
[0x1D] Sending heartbeat acknowledgment

[PACKET] Opcode: 0x25, Size: 36 bytes
[0x25] ENCRYPTED PACKET - attempting decryption
[SUCCESS] Decrypted 24 bytes!
[HEXDUMP] DECRYPTED DATA (24 bytes):
  0000  34 10 01 00 00 00 75 73  65 72 6e 61 6d 65 00 70  |4.....username.p|
  0010  61 73 73 77 6f 72 64 00                           |assword.|

!!! GAME MESSAGE OPCODE: 0x1034 !!!
!!! THIS IS A GAME MESSAGE (0x1000+) !!!
```

### Key Indicators of Success

1. **Flash Policy Exchange** - Client sends `<policy-file-request/>`, server responds with XML
2. **Encryption Handshake** - Server sends 0x04 with RSA public key
3. **Session Key Exchange** - Client sends 0x05, server decrypts AES key successfully
4. **Encrypted Messages** - Server successfully decrypts 0x25 packets and extracts opcodes

### Expected Game Message Opcodes

Based on the message catalog, you should see opcodes like:

- **0x1034** - ReqLogin (login request with username/password)
- **0x1035** - AnsLogin (login response)
- **0x1036** - ReqCharNameList (character list request)
- **0x1037** - AnsCharNameList (character list response)
- **0x1038+** - Various game messages

## Troubleshooting

### Client Doesn't Connect

1. Check firewall - port 7101 must be open
2. Verify hosts file was modified correctly
3. Try connecting from same machine first (localhost)
4. Check if another service is using port 7101

### Connection Drops Immediately

1. Client might be checking server version/signature
2. Check test server logs for error messages
3. The client might expect specific responses we haven't implemented

### Decryption Fails

1. Check if AES key was decrypted successfully in 0x05 handler
2. Verify RSA keypair generation worked
3. Look for errors in the decryption log output

### No 0x25 Packets

1. Client might have disconnected before sending game messages
2. Check if heartbeat (0x1B/0x1D) exchange is working
3. Client might need specific responses before proceeding

## Capturing Opcodes

As encrypted packets are decrypted, the server will output:

```
!!! GAME MESSAGE OPCODE: 0xXXXX !!!
```

**Create a list of all opcodes you see** - this is critical data for implementing game handlers!

### Recommended: Log to File

```bash
cargo run --bin test_server 2>&1 | tee test_server.log
```

Then extract opcodes:
```bash
grep "GAME MESSAGE OPCODE" test_server.log | sort -u
```

## Next Steps

Once you've captured opcodes from decrypted packets:

1. **Map Opcodes to Messages**
   - Cross-reference with `docs/protocol/appendices/message-catalog.md`
   - Identify which opcode = which message (e.g., 0x1034 = ReqLogin)

2. **Implement Game Handlers**
   - Start with login flow (ReqLogin, AnsLogin)
   - Add character list handlers
   - Expand to other game features

3. **Update MessageType Enum**
   - Replace placeholder opcodes with real values
   - Add newly discovered messages

4. **Document Protocol**
   - Update `PACKET-CAPTURE-ANALYSIS.md` with new findings
   - Create detailed message structure documentation

## Example Session Log

See `docs/testing/EXAMPLE-TEST-SESSION.md` (to be created after first successful test).

## Known Limitations

1. **Server doesn't implement game logic** - it only logs packets
2. **Client will disconnect** after timeout if no proper responses
3. **Some messages might not be sent** if prerequisites aren't met
4. **ProudNet settings are hardcoded** - might need adjustment for different client versions

## Safety Notes

- This is a **test server** - not a production server
- It logs **all decrypted data** including passwords (if sent)
- Use only with test accounts
- Don't expose port 7101 to the internet
