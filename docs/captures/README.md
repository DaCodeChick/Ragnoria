# Packet Captures Directory

This directory stores network traffic captures from the Ragnarok Online 2 client for analysis.

## Files

Place your `.pcapng` or `.txt` (hex dump) files here for analysis with the packet analyzer tool.

### Recommended Captures

1. **`login_flow.pcapng`** - Complete login sequence (client → login server → lobby)
2. **`character_select.pcapng`** - Character list and selection
3. **`world_enter.pcapng`** - Entering the game world
4. **`gameplay_sample.pcapng`** - Movement, combat, chat samples

## Usage

After capturing traffic with Wireshark (see `../CAPTURE-GUIDE.md`), analyze with:

```bash
# Analyze a hex dump file
cargo run --bin packet-analyzer -- file docs/captures/login_flow_hex.txt

# Analyze specific hex bytes
cargo run --bin packet-analyzer -- hex --data "50524F5500..."
```

## Security Note

**⚠️ WARNING:** Capture files may contain sensitive information (usernames, passwords, session keys).

- **DO NOT** commit captures to version control (this directory is in `.gitignore`)
- **DO NOT** share captures publicly
- Use test accounts only when capturing traffic

## See Also

- `../CAPTURE-GUIDE.md` - How to capture packets with Wireshark
- `EXAMPLE-ANALYSIS.md` - Complete analysis walkthrough
