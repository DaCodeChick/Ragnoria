#!/bin/bash
# Script to capture official RO2 login server traffic
# This will help us understand the exact 0x0000 handshake sequence

set -e

CAPTURE_FILE="captures/ro2login_new_$(date +%Y%m%d_%H%M%S).pcapng"
CAPTURE_DIR="captures"

echo "=========================================="
echo "RO2 Official Server Traffic Capture"
echo "=========================================="
echo ""

# Create captures directory if it doesn't exist
mkdir -p "$CAPTURE_DIR"

# Detect the official server IP (you'll need to update this)
OFFICIAL_SERVER_IP="129.241.93.210"  # Update this if needed
OFFICIAL_SERVER_PORT="7101"

echo "Configuration:"
echo "  Official Server: $OFFICIAL_SERVER_IP:$OFFICIAL_SERVER_PORT"
echo "  Capture File: $CAPTURE_FILE"
echo ""

echo "INSTRUCTIONS:"
echo "============="
echo ""
echo "1. This script will start tcpdump to capture traffic"
echo "2. Launch RO2 client and connect to OFFICIAL server"
echo "3. Enter credentials and click Connect"
echo "4. Wait for login to complete (or timeout)"
echo "5. Press Ctrl+C to stop capture"
echo ""
echo "IMPORTANT: Make sure you're connecting to the OFFICIAL server,"
echo "           not localhost! Check your hosts file."
echo ""
echo "Press Enter to start capture..."
read

# Build tcpdump filter
FILTER="host $OFFICIAL_SERVER_IP and tcp port $OFFICIAL_SERVER_PORT"

echo ""
echo "Starting tcpdump..."
echo "Filter: $FILTER"
echo ""
echo "Capture will be saved to: $CAPTURE_FILE"
echo "Press Ctrl+C when done."
echo ""

# Run tcpdump (needs sudo)
sudo tcpdump -i any -s 65535 -w "$CAPTURE_FILE" "$FILTER"

echo ""
echo "=========================================="
echo "Capture complete!"
echo "=========================================="
echo ""
echo "File saved: $CAPTURE_FILE"
echo ""
echo "Quick analysis:"
echo "  Total packets: $(tshark -r "$CAPTURE_FILE" 2>/dev/null | wc -l)"
echo ""
echo "Next steps:"
echo "  1. Review capture: tshark -r $CAPTURE_FILE"
echo "  2. Find 0x0000 packets: Look for 40-byte packets after handshake"
echo "  3. Extract decrypted payloads using ProudNet session keys"
echo ""
