#!/usr/bin/env python3
"""
Analyze the 0x0000 packet structure from official server PCAP
"""

# From PCAP analysis:
# Frame 18: Client → Server (0x0000 request)
frame18_full = (
    "1357012425010120cad0485b90a0baa28eeceaac1fddc07427813f0a69515e4a7cd1bb5712f8846a"
)

# Frame 20: Server → Client (0x0000 response)
frame20_full = (
    "1357012425010120d5fe7c371251bb19d1601b5aff2f2b8cc93cbcdd4918bcacb85fb264e0e7b2ab"
)


def parse_encrypted_packet(hex_str, label):
    """Parse ProudNet 0x25 encrypted packet"""
    data = bytes.fromhex(hex_str)

    print(f"\n{'=' * 70}")
    print(f"{label}")
    print(f"{'=' * 70}")
    print(f"Raw hex: {hex_str}\n")

    # ProudNet frame structure:
    # [0-1] Magic (0x1357)
    # [2] Varint size byte
    # [3] Varint value (payload length)
    # [4+] Payload

    magic = data[0:2]
    varint_size = data[2]
    payload_len = data[3]  # Since varint_size is 1, only 1 byte for length

    print(f"Magic: {magic.hex()} (should be 1357)")
    print(f"Varint size byte: {varint_size}")
    print(f"Payload length: {payload_len} (0x{payload_len:02X})")
    print(f"Actual packet length: {len(data)} bytes")
    print(
        f"Expected total: 2 (magic) + 1 (varint size) + 1 (length) + {payload_len} (payload) = {4 + payload_len}"
    )

    # Extract payload
    payload = data[4 : 4 + payload_len]
    print(f"\nPayload ({len(payload)} bytes):")

    # Parse 0x25 encrypted payload structure
    opcode = payload[0]
    enc_type = payload[1]
    msg_type = payload[2]
    enc_len = payload[3]
    encrypted_data = payload[4:]

    print(f"  Opcode: 0x{opcode:02X} (should be 0x25 for encrypted)")
    print(f"  Encryption type: 0x{enc_type:02X}")
    print(f"  Message type: 0x{msg_type:02X}")
    print(f"  Encrypted data length: {enc_len} (0x{enc_len:02X})")
    print(f"  Actual encrypted data: {len(encrypted_data)} bytes")

    print(f"\n  Encrypted data (hex):")
    for i in range(0, len(encrypted_data), 16):
        chunk = encrypted_data[i : i + 16]
        hex_str = " ".join(f"{b:02x}" for b in chunk)
        print(f"    [{i:2d}] {hex_str}")

    return encrypted_data


# Parse both packets
client_encrypted = parse_encrypted_packet(
    frame18_full, "Frame 18: Client 0x0000 Request"
)
server_encrypted = parse_encrypted_packet(
    frame20_full, "Frame 20: Server 0x0000 Response"
)

print(f"\n{'=' * 70}")
print("COMPARISON")
print(f"{'=' * 70}")
print(f"Client encrypted length: {len(client_encrypted)}")
print(f"Server encrypted length: {len(server_encrypted)}")
print(f"Are they the same length? {len(client_encrypted) == len(server_encrypted)}")

print(f"\n{'=' * 70}")
print("PROBLEM STATEMENT")
print(f"{'=' * 70}")
print("""
The official server sends back 33 bytes of AES-encrypted data in response
to the client's 0x0000 handshake. This encrypted data decrypts to a 26-byte
0x0000 response packet.

Our server ALSO sends 0x0000 responses, but the client doesn't send the
0x2EE2 login packet afterward. This suggests:

1. The decrypted content is DIFFERENT from what the official server sends
2. There's a validation check in the client that fails on our response
3. The mysterious bytes 0x803F0000 at offset 0x16-0x19 might be wrong

WITHOUT THE AES KEY, we cannot decrypt the PCAP packets to see what the
official server actually sent.

NEXT STEPS:
1. Log what OUR server is sending (decrypted payload)
2. Try modifying suspicious bytes (especially 0x803F0000)
3. Use dynamic analysis (x64dbg breakpoint on SendReqLogin)
""")
