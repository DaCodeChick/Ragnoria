#!/bin/bash
# Debug the RO2 client to see where it exits

CLIENT_PATH="/run/media/admin/FE6407F46407AE89/Gravity/Ragnarok Online 2 - Jawaii/SHIPPING/Rag2.exe"

cd "$(dirname "$CLIENT_PATH")"

echo "Starting client under winedbg..."
echo "Commands you can use:"
echo "  bt - backtrace (show call stack)"
echo "  c - continue execution"
echo "  q - quit debugger"
echo ""
echo "Setting breakpoints at our patched functions..."
echo ""

# Create a winedbg command file
cat > /tmp/winedbg_commands.txt <<'EOF'
# Break at our patched functions to verify they're being called
break *0x00A4FFA0
break *0x00A4CEF0

# Break at common exit points
break ExitProcess

# Continue execution
c
EOF

# Run with debugger
WINEDEBUG=-all winedbg --command "$(cat /tmp/winedbg_commands.txt)" "$CLIENT_PATH"
