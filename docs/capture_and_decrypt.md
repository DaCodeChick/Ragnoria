# Capturing and Decrypting Client Traffic

## Goal
Capture traffic between the RO2 client and OUR server, then decrypt it using the logged AES session keys to see exactly what the client is sending/expecting.

## Prerequisites

```bash
# Install Python crypto library
pip3 install pycryptodome

# Or system package
sudo apt-get install python3-pycryptodome
```

## Step 1: Start Server with Key Logging

The server now logs AES session keys with a 🔑 emoji prefix.

```bash
cd /home/admin/Documents/GitHub/Ragnoria

# Kill old server
sudo lsof -ti:7101 | xargs -r sudo kill -9

# Start new server (it will log keys)
RUST_LOG=info cargo run -p ro2-login 2>&1 | tee /tmp/ro2_server_$(date +%H%M%S).log &
echo $! > /tmp/server.pid
```

## Step 2: Start Packet Capture

In a **separate terminal**, start capturing:

```bash
cd /home/admin/Documents/GitHub/Ragnoria

# Create captures directory
mkdir -p captures

# Start capture
CAPTURE_FILE="captures/client_$(date +%Y%m%d_%H%M%S).pcapng"
sudo tcpdump -i lo -s 65535 -w "$CAPTURE_FILE" "port 7101"
```

## Step 3: Trigger Client Connection

In a **third terminal** or from GUI:

```bash
# Launch client
cargo run --bin launcher
# Click "Launch Game"
# Enter credentials and click "Connect"
# Wait for timeout
```

## Step 4: Stop Capture

After client times out, press **Ctrl+C** in the tcpdump terminal.

## Step 5: Extract Session Key from Logs

```bash
# Find the session key in server logs
grep "AES_SESSION_KEY" /tmp/ro2_server_*.log

# Example output:
# 🔑 AES_SESSION_KEY [127.0.0.1:52748]: a1b2c3d4e5f67890123456789abcdef0
```

Copy the hex key (32 characters = 16 bytes).

## Step 6: Extract Packets from PCAP

```bash
# Get all TCP data
tshark -r "$CAPTURE_FILE" -Y "tcp.port == 7101" \
  -T fields -e frame.number -e ip.src -e data.data 2>/dev/null | \
  grep -v "^[0-9]*$" > /tmp/packets.txt

# View the file
cat /tmp/packets.txt
```

## Step 7: Decrypt Specific Packets

Find the packets you want to decrypt (look for opcode 0x25):

```bash
# Decrypt a packet
./tools/decrypt_packet.py <PACKET_HEX> <SESSION_KEY_HEX>

# Example:
./tools/decrypt_packet.py \
  "1357012425010120db78e73458c7edd03fb68b8e773183e2066e32df78678063ea8fa7c49375f63d" \
  "a1b2c3d4e5f67890123456789abcdef0"
```

## Step 8: Find the 0x0000 Exchange

Look for:
1. **Client → Server**: First encrypted packet after 0x06 (encryption ready)
2. **Server → Client**: Our response
3. **Client → Server**: Expected to be 0x2EE2 (login), but might be something else

```bash
# Filter for encrypted packets (opcode 0x25)
grep "25" /tmp/packets.txt

# The first few 0x25 packets will be the 0x0000 exchange
```

## Step 9: Analyze Decrypted Data

The decryption tool will show:
- **Game opcode** (e.g., 0x0000, 0x2EE2)
- **Full decrypted payload** in hex
- **Payload length**

Compare:
- What the client sends for 0x0000
- What we respond with
- What the client sends next (or doesn't send)

## Example Workflow

```bash
# Terminal 1: Server
RUST_LOG=info cargo run -p ro2-login 2>&1 | tee /tmp/server.log &

# Terminal 2: Capture
sudo tcpdump -i lo -w captures/test.pcapng port 7101

# Terminal 3: Client
cargo run --bin launcher
# (click through GUI)

# After test:
# Terminal 2: Ctrl+C to stop capture

# Extract session key
SESSION_KEY=$(grep "AES_SESSION_KEY" /tmp/server.log | tail -1 | awk '{print $NF}')
echo "Session Key: $SESSION_KEY"

# Extract packets
tshark -r captures/test.pcapng -Y "tcp.port == 7101" \
  -T fields -e frame.number -e tcp.srcport -e data.data 2>/dev/null > /tmp/packets.txt

# Find client's 0x0000 (first packet from high port after encryption)
CLIENT_PORT=$(tshark -r captures/test.pcapng -Y "tcp.port == 7101 and tcp.flags.push == 1" \
  -T fields -e tcp.srcport 2>/dev/null | head -1)

echo "Client port: $CLIENT_PORT"

# Get packets from client (srcport == CLIENT_PORT, opcode 0x25)
grep "^[0-9]*\t$CLIENT_PORT" /tmp/packets.txt | grep "25" | head -3

# Decrypt each packet
# (manually copy hex and run decrypt script)
```

## What We're Looking For

1. **Client's 0x0000 packet** (decrypted payload should match what we know)
2. **Our server's 0x0000 response** (is it correct?)
3. **Client's next packet** (is it 0x2EE2 or something else?)
4. **Any error packets from client** (rejection, disconnect reason)

## Expected Results

**Success case:**
```
Frame 10 (Client → Server): 0x0000 with 26-byte payload
Frame 11 (Server → Client): 0x0000 response with 26-byte payload
Frame 12 (Client → Server): 0x2EE2 with 211-byte payload (LOGIN!)
```

**Failure case (current):**
```
Frame 10 (Client → Server): 0x0000 with 26-byte payload
Frame 11 (Server → Client): 0x0000 response with 26-byte payload
Frame 12 (Client → Server): 0x1C (keep-alive)
Frame 13 (Client → Server): 0x1B (heartbeat)
... (no 0x2EE2)
Frame 50 (Client → Server): 0x01 (disconnect)
```

This will tell us **exactly** why the client is rejecting our response!

## Troubleshooting

### Can't decrypt packets
- Check session key is exactly 32 hex characters
- Make sure packet hex starts with ProudNet magic (1357...)
- Verify opcode is 0x25 (encrypted)

### Python import error
```bash
pip3 install pycryptodome
# or
pip3 install pycrypto
```

### Can't find packets in PCAP
```bash
# List all frames
tshark -r captures/test.pcapng -Y "tcp.port == 7101" 2>/dev/null

# Check if capture is empty
tshark -r captures/test.pcapng 2>/dev/null | wc -l
```

---

**Ready to start?** Run the commands above and share the decrypted 0x0000 exchange!
