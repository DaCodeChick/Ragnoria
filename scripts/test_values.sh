#!/bin/bash
# RO2 Login Server Test Script
# Tests different values for the mysterious 0x803F0000 field

echo "=========================================="
echo "RO2 Login Server - Byte Fuzzing Test"
echo "=========================================="
echo ""

# Test values for bytes at offset 0x16-0x19
TEST_VALUES=(
    "0x00, 0x00, 0x80, 0x3F"  # float 1.0 LE (CURRENT TEST)
    "0x80, 0x3F, 0x00, 0x00"  # Client's value (mirror)
    "0x3F, 0x80, 0x00, 0x00"  # float 1.0 BE
    "0x00, 0x00, 0x00, 0x00"  # All zeros
    "0xFF, 0xFF, 0xFF, 0xFF"  # All ones
    "0x01, 0x00, 0x00, 0x00"  # Integer 1 LE
    "0x00, 0x00, 0x00, 0x01"  # Integer 1 BE
)

TEST_DESCRIPTIONS=(
    "Float 1.0 in correct little-endian format"
    "Mirror client's exact value (original behavior)"
    "Float 1.0 in big-endian format"
    "All zeros (disabled flag?)"
    "All ones (enabled flag?)"
    "Integer 1 in little-endian"
    "Integer 1 in big-endian"
)

echo "Current test: ${TEST_DESCRIPTIONS[0]}"
echo "Value: ${TEST_VALUES[0]}"
echo ""
echo "If this doesn't work, edit ro2-login/src/main.rs line ~369"
echo "and change the client_field4 value to one of these:"
echo ""

for i in "${!TEST_VALUES[@]}"; do
    echo "Test $((i+1)): ${TEST_DESCRIPTIONS[$i]}"
    echo "  let client_field4 = [${TEST_VALUES[$i]}];"
    echo ""
done

echo "=========================================="
echo "Quick Test Instructions:"
echo "=========================================="
echo "1. Client should be connected now"
echo "2. Try logging in"
echo "3. If timeout occurs again:"
echo "   - Stop server: pkill ro2-login"
echo "   - Edit the value in main.rs"
echo "   - Rebuild: cargo build --release --bin ro2-login"
echo "   - Re-run this test"
echo ""
echo "To see what we sent:"
echo "  cat /tmp/ro2-test.log | grep 'Full hex'"
