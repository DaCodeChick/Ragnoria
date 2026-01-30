#!/usr/bin/env python3
"""
Decrypt ProudNet AES-encrypted packets from PCAP using logged session keys.

Usage:
    1. Start server (it will log AES session keys with 🔑 emoji)
    2. Capture traffic: sudo tcpdump -i any -w capture.pcapng port 7101
    3. Extract session key from logs
    4. Run this script: ./decrypt_capture.py capture.pcapng <session_key_hex>
"""

import sys
import binascii
from Crypto.Cipher import AES


def decrypt_aes_ecb(ciphertext: bytes, key: bytes) -> bytes:
    """Decrypt using AES-128-ECB (as used by ProudNet)."""
    cipher = AES.new(key, AES.MODE_ECB)
    return cipher.decrypt(ciphertext)


def parse_proudnet_packet(data: bytes) -> dict:
    """Parse ProudNet packet structure."""
    if len(data) < 5:
        return {"error": "Packet too short"}

    # ProudNet frame structure:
    # 0x00-0x03: Magic (0x13570124 or similar)
    # 0x04: Opcode
    # 0x05+: Payload

    magic = int.from_bytes(data[0:4], "little")
    opcode = data[4]
    payload = data[5:]

    result = {
        "magic": f"0x{magic:08x}",
        "opcode": f"0x{opcode:02x}",
        "payload_len": len(payload),
    }

    # If opcode is 0x25 (encrypted game message)
    if opcode == 0x25:
        if len(payload) < 3:
            result["error"] = "0x25 payload too short"
            return result

        # 0x25 structure:
        # 0x00-0x01: Flags/sequence
        # 0x02: Encrypted data length
        # 0x03+: Encrypted data

        flags = int.from_bytes(payload[0:2], "little")
        enc_len = payload[2]
        encrypted = payload[3 : 3 + enc_len]

        result["flags"] = f"0x{flags:04x}"
        result["encrypted_len"] = enc_len
        result["encrypted_data"] = binascii.hexlify(encrypted).decode()

    return result


def decrypt_packet(packet_hex: str, session_key_hex: str) -> dict:
    """Decrypt a single ProudNet encrypted packet."""
    try:
        packet_data = binascii.unhexlify(packet_hex)
        session_key = binascii.unhexlify(session_key_hex)

        if len(session_key) != 16:
            return {"error": f"Session key must be 16 bytes, got {len(session_key)}"}

        parsed = parse_proudnet_packet(packet_data)

        if parsed.get("opcode") == "0x25":
            encrypted_hex = parsed.get("encrypted_data")
            if encrypted_hex:
                encrypted = binascii.unhexlify(encrypted_hex)

                # Decrypt using AES-128-ECB
                decrypted = decrypt_aes_ecb(encrypted, session_key)

                # Parse decrypted payload
                if len(decrypted) >= 2:
                    game_opcode = int.from_bytes(decrypted[0:2], "little")
                    parsed["decrypted"] = {
                        "game_opcode": f"0x{game_opcode:04x}",
                        "payload": binascii.hexlify(decrypted).decode(),
                        "payload_bytes": len(decrypted),
                    }

        return parsed
    except Exception as e:
        return {"error": str(e)}


def main():
    if len(sys.argv) < 3:
        print("Usage: ./decrypt_packet.py <packet_hex> <session_key_hex>")
        print("")
        print("Example:")
        print("  ./decrypt_packet.py 1357012425010120db78e73458c7ed... a1b2c3d4e5f6...")
        print("")
        print("To extract packets from PCAP:")
        print("  tshark -r capture.pcapng -Y 'tcp.port == 7101' \\")
        print("    -T fields -e data.data | grep -v '^$'")
        return 1

    packet_hex = sys.argv[1]
    session_key_hex = sys.argv[2]

    print("=" * 60)
    print("ProudNet Packet Decryption")
    print("=" * 60)
    print(f"Session Key: {session_key_hex}")
    print(
        f"Packet Data: {packet_hex[:60]}..."
        if len(packet_hex) > 60
        else f"Packet Data: {packet_hex}"
    )
    print("")

    result = decrypt_packet(packet_hex, session_key_hex)

    print("Parsed Packet:")
    print("-" * 60)
    for key, value in result.items():
        if isinstance(value, dict):
            print(f"  {key}:")
            for k2, v2 in value.items():
                print(f"    {k2}: {v2}")
        else:
            print(f"  {key}: {value}")

    print("=" * 60)

    return 0


if __name__ == "__main__":
    sys.exit(main())
