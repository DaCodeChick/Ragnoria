# How to Capture and Analyze Official Server Traffic

## Prerequisites

1. Access to the official RO2 server (or a working private server)
2. tcpdump/tshark installed
3. sudo access for packet capture

## Step 1: Prepare for Capture

### Check Current Server Configuration

First, verify which server your client is connecting to:

```bash
# Check hosts file
cat /etc/hosts | grep gnjoy

# If it points to localhost, comment it out temporarily:
sudo nano /etc/hosts
# Comment out or remove the line: 127.0.0.1 gjnoy.cloudapp.net
```

### Identify Official Server IP

```bash
# Find the real server IP
nslookup gnjoy.cloudapp.net
# Or
dig gnjoy.cloudapp.net
```

## Step 2: Start Packet Capture

### Option A: Use the provided script

```bash
cd /home/admin/Documents/GitHub/Ragnoria
chmod +x docs/capture_official_server.sh
./docs/capture_official_server.sh
```

### Option B: Manual capture

```bash
# Replace OFFICIAL_IP with the real server IP
OFFICIAL_IP="129.241.93.210"
CAPTURE_FILE="captures/ro2login_$(date +%Y%m%d_%H%M%S).pcapng"

sudo tcpdump -i any -s 65535 -w "$CAPTURE_FILE" \
  "host $OFFICIAL_IP and tcp port 7101"
```

## Step 3: Trigger Login Sequence

While tcpdump is running:

1. **Launch RO2 Client**
   ```bash
   cargo run --bin launcher
   # Click "Launch Game"
   ```

2. **Enter credentials** and click **Connect**

3. **Let the login complete** (or timeout)

4. **Stop capture** with Ctrl+C

## Step 4: Analyze the Capture

### Quick Overview

```bash
CAPTURE_FILE="captures/ro2login_TIMESTAMP.pcapng"

# Count total packets
tshark -r "$CAPTURE_FILE" 2>/dev/null | wc -l

# View packet summary
tshark -r "$CAPTURE_FILE" -Y "tcp.port == 7101" 2>/dev/null

# Find packets by size (0x0000 handshake is typically 40 bytes)
tshark -r "$CAPTURE_FILE" -Y "tcp.len == 40" 2>/dev/null
```

### Identify the 0x0000 Handshake

The 0x0000 handshake happens right after ProudNet encryption is established:

```bash
# Look for the sequence:
# 1. Client sends 40-byte packet (encrypted 0x0000)
# 2. Server sends 40-byte packet (response)
# 3. Client sends 234-byte packet (0x2EE2 login)

tshark -r "$CAPTURE_FILE" -Y "tcp.port == 7101" \
  -T fields -e frame.number -e frame.time_relative -e tcp.len -e data.data 2>/dev/null
```

### Extract Specific Packets

Find the critical packets:

```bash
# Find first 40-byte packet (client 0x0000)
tshark -r "$CAPTURE_FILE" -Y "tcp.len == 40 and ip.src == YOUR_CLIENT_IP" \
  -T fields -e frame.number 2>/dev/null | head -1

# Find the corresponding server response
# Should be the next 40-byte packet from server
```

### Get Hex Dumps

```bash
# Get hex dump of specific frame (replace FRAME_NUM)
tshark -r "$CAPTURE_FILE" -Y "frame.number == FRAME_NUM" -x 2>/dev/null

# Or extract just the payload
tshark -r "$CAPTURE_FILE" -Y "frame.number == FRAME_NUM" \
  -T fields -e data.data 2>/dev/null
```

## Step 5: Extract ProudNet Session Keys (Advanced)

The packets are AES-encrypted. To decrypt them, we need the session keys from the handshake.

### Find the RSA Key Exchange

```bash
# Look for the initial handshake (opcode 0x04)
# This contains the encrypted AES session key

tshark -r "$CAPTURE_FILE" -Y "tcp.port == 7101" \
  -T fields -e frame.number -e tcp.len 2>/dev/null | \
  awk '$2 > 100 && $2 < 300' | head -5
```

The client sends an RSA-encrypted AES key in the early handshake. Without the server's private RSA key, we can't decrypt this.

## Step 6: Compare Packet Patterns

Even without decryption, we can compare:

1. **Packet sizes** - Does official server send same sizes?
2. **Timing** - How long between 0x0000 and 0x2EE2?
3. **Packet count** - Does server send extra messages?

```bash
# Create a timing analysis
tshark -r "$CAPTURE_FILE" -Y "tcp.port == 7101 and tcp.len > 0" \
  -T fields -e frame.time_relative -e ip.src -e tcp.len 2>/dev/null | \
  awk '{printf "%8.3f  %-15s  %4d bytes\n", $1, $2, $3}'
```

## Step 7: Look for Patterns

### Compare with Old Capture

```bash
# Old capture
tshark -r captures/ro2login.pcapng -Y "tcp.port == 7101" \
  -T fields -e frame.number -e tcp.len 2>/dev/null > /tmp/old_packets.txt

# New capture  
tshark -r "$CAPTURE_FILE" -Y "tcp.port == 7101" \
  -T fields -e frame.number -e tcp.len 2>/dev/null > /tmp/new_packets.txt

# Compare
diff -u /tmp/old_packets.txt /tmp/new_packets.txt
```

## What We're Looking For

1. **Server's 0x0000 response structure**
   - Exact encrypted payload (even if we can't decrypt)
   - Timing relative to client's 0x0000
   - Any additional messages sent

2. **Confirmation of sequence**
   - Does server send exactly ONE response?
   - Or multiple responses?
   - What sizes are they?

3. **Client's reaction**
   - How quickly does it send 0x2EE2?
   - Are there any intermediate messages?

## Alternative: Try to Decrypt

If you have Wireshark with SSL/TLS dissector plugins:

1. Open the capture in Wireshark
2. Try to export SSL session keys if available
3. Use custom dissector for ProudNet protocol

Or, we could try:

1. **Modify our server** to log the AES session keys
2. **Capture traffic to OUR server**
3. **Use logged keys to decrypt in Wireshark**

This would let us see exactly what the client is sending/expecting!

## Next Steps After Analysis

Once you have the capture:

1. Share the file or key findings
2. We'll compare with our current implementation
3. Adjust our 0x0000 response to match exactly
4. Test again!

---

**TIP**: If the official server is down or you can't capture from it, we can instead:
- Capture traffic between the client and OUR server
- Log the AES session keys from our server
- Decrypt the packets ourselves to see what the client expects
