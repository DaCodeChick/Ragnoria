# Ghidra Analysis Documentation - Rag2.exe

This directory contains reverse engineering documentation from analyzing the Ragnarok Online 2 client executable (Rag2.exe) using Ghidra.

## Analysis Date
January 31, 2026

## Binary Information
- **File**: Rag2.exe
- **Path**: `/C:/Gravity/Ragnarok Online 2 - Jawaii/SHIPPING/Rag2.exe`
- **Format**: Portable Executable (PE) - 32-bit x86
- **Platform**: Windows (running under Wine on Linux)
- **Architecture**: x86:LE:32:default (x86 Little-Endian 32-bit)

## Analysis Tools
- **Primary**: Ghidra (via ghidra-bridge API)
- **Secondary**: x64dbg (planned for dynamic analysis)
- **Packet Analysis**: Wireshark with ro2game2.pcapng

## Key Findings Summary
See [FINDINGS.md](../../FINDINGS.md) for complete analysis.

## Contents
- [function_analysis.md](function_analysis.md) - Detailed function documentation
- [packet_structures.md](packet_structures.md) - Network packet format specifications
- [opcode_reference.md](opcode_reference.md) - Complete opcode mapping

---

## Quick Reference

### Critical Functions
| Function | Address | Purpose |
|----------|---------|---------|
| DispatchLoginAuthPackets | 0x00E552E0 | Routes login request packets |
| DispatchAckPackets_0x30D5_0x30DC | 0x00E58940 | Routes server response packets |
| SendReqLogin | 0x00E52FE0 | Builds and sends login packet |
| DispatchProudNetProtocolPackets | 0x00F43FF0 | ProudNet protocol layer |

### Critical Opcodes
| Opcode | Name | Direction | Size | Description |
|--------|------|-----------|------|-------------|
| 0x0000 | InitialHandshake | Both | 26 bytes | Mystery handshake packet |
| 0x2EE2 | ReqLogin | C→S | 211 bytes | Login authentication request |
| 0x30D5 | AckLogin | S→C | 82 bytes | Login authentication response |
| 0x30D4 | AckVersionCheck | S→C | Variable | Version validation response |

### Mystery Bytes Under Investigation
| Offset | Value | Purpose | Status |
|--------|-------|---------|--------|
| 0x16-0x19 | 0x803F0000 | Unknown | SUSPECTED CAUSE |
| 0x12-0x15 | 0x07022500 | Unknown | Under investigation |
| 0x0E-0x11 | 0x00000001 | Status/Flags | Known (must be 0x01) |

---

## Analysis Methodology

1. **Function Discovery**: Used pattern matching to find login-related functions
2. **Decompilation**: Ghidra's decompiler to understand logic
3. **Cross-Reference Tracing**: Tracked data/function usage
4. **String Analysis**: Located error messages and packet names
5. **Byte Pattern Search**: Found suspicious constants (0x803F0000)
6. **PCAP Comparison**: Matched decompiled code to captured network traffic

## Limitations
- Cannot locate 0x0000 handler (likely obfuscated or inlined)
- SendReqLogin has no direct callers (virtual function/COM interface)
- UI layer uses COM/DirectX callbacks (hard to trace)
- Some optimizations make code hard to follow

## Next Steps
1. Capture official server PCAP with successful login
2. Compare byte-for-byte with our implementation
3. Dynamic analysis with x64dbg if needed
4. Field fuzzing if comparison doesn't reveal issue
