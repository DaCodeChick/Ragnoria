#!/bin/bash
# Quick test script: capture and analyze client traffic

set -e

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_FILE="/tmp/ro2_server_$TIMESTAMP.log"
CAPTURE_FILE="captures/client_$TIMESTAMP.pcapng"

echo "=========================================="
echo "RO2 Client Traffic Capture & Analysis"
echo "=========================================="
echo ""
echo "Log: $LOG_FILE"
echo "Capture: $CAPTURE_FILE"
echo ""

# Kill old server
echo "[1/5] Stopping old server..."
sudo lsof -ti:7101 2>/dev/null | xargs -r sudo kill -9 || true
sleep 1

# Create captures directory
mkdir -p captures

# Start capture in background
echo "[2/5] Starting tcpdump..."
sudo tcpdump -i lo -s 65535 -w "$CAPTURE_FILE" "port 7101" &
TCPDUMP_PID=$!
echo "  tcpdump PID: $TCPDUMP_PID"
sleep 2

# Start server in background
echo "[3/5] Starting server..."
cd /home/admin/Documents/GitHub/Ragnoria
RUST_LOG=info cargo run -p ro2-login 2>&1 | tee "$LOG_FILE" &
SERVER_PID=$!
echo "  Server PID: $SERVER_PID"
echo $SERVER_PID > /tmp/server.pid
sleep 3

echo ""
echo "=========================================="
echo "READY FOR CLIENT CONNECTION"
echo "=========================================="
echo ""
echo "Now launch the RO2 client:"
echo "  1. Run: cargo run --bin launcher"
echo "  2. Click 'Launch Game'"
echo "  3. Enter credentials and click 'Connect'"
echo "  4. Wait for timeout"
echo ""
echo "Press Enter when client has disconnected..."
read

# Stop capture
echo ""
echo "[4/5] Stopping capture..."
sudo kill -INT $TCPDUMP_PID 2>/dev/null || true
sleep 2

# Stop server
echo "[5/5] Stopping server..."
kill $SERVER_PID 2>/dev/null || true
sleep 1

echo ""
echo "=========================================="
echo "ANALYSIS"
echo "=========================================="
echo ""

# Extract session key
echo "Session Key:"
SESSION_KEY=$(grep "AES_SESSION_KEY" "$LOG_FILE" | tail -1 | awk -F'[][]' '{print $2}' | awk '{print $NF}')
if [ -n "$SESSION_KEY" ]; then
    echo "  $SESSION_KEY"
    echo "  (saved to /tmp/session_key.txt)"
    echo "$SESSION_KEY" > /tmp/session_key.txt
else
    echo "  ⚠️  NOT FOUND - check if client connected"
fi
echo ""

# Count packets
PACKET_COUNT=$(tshark -r "$CAPTURE_FILE" -Y "tcp.port == 7101" 2>/dev/null | wc -l)
echo "Total packets captured: $PACKET_COUNT"
echo ""

if [ "$PACKET_COUNT" -gt 0 ]; then
    echo "Packet summary:"
    tshark -r "$CAPTURE_FILE" -Y "tcp.port == 7101 and tcp.len > 0" \
      -T fields -e frame.number -e tcp.srcport -e tcp.len 2>/dev/null | \
      head -20
    echo ""
    
    echo "To decrypt packets:"
    echo "  1. Extract packet hex:"
    echo "     tshark -r '$CAPTURE_FILE' -Y 'frame.number == N' -T fields -e data.data"
    echo ""
    echo "  2. Decrypt:"
    echo "     ./tools/decrypt_packet.py <PACKET_HEX> $SESSION_KEY"
    echo ""
fi

echo "Files:"
echo "  Server log: $LOG_FILE"
echo "  Capture: $CAPTURE_FILE"
echo "  Session key: /tmp/session_key.txt"
echo ""
echo "=========================================="
