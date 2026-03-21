#!/usr/bin/env python3
"""
Decrypt ProudNet encrypted packets from RO2 PCAP
"""

from Crypto.Cipher import AES
from Crypto.Util.Padding import unpad
import binascii

# ProudNet packet structure:
# 0x1357 (2 bytes) - Magic header
# Length (2 bytes) - Total packet length including header
# Opcode (1 byte) - ProudNet opcode (0x25 = encrypted)
# Payload...


def decrypt_proudnet_packet(hex_data: str, aes_key: bytes) -> bytes:
    """Decrypt a ProudNet 0x25 encrypted packet"""
    data = bytes.fromhex(hex_data)

    # Parse ProudNet header
    magic = data[0:2]
    length = int.from_bytes(data[2:4], "little")
    opcode = data[4]

    print(f"  Magic: {magic.hex()}")
    print(f"  Length: {length}")
    print(f"  Opcode: 0x{opcode:02X}")

    if opcode != 0x25:
        print(f"  ERROR: Not an encrypted packet (0x25), got 0x{opcode:02X}")
        return None

    # Parse 0x25 encrypted packet structure:
    # 0x25 (1 byte) - opcode
    # 0x01 (1 byte) - encryption type (0x01 = AES)
    # 0x01 (1 byte) - message type
    # 0x20 (1 byte) - IV length (32 bytes)
    # IV (32 bytes)
    # Encrypted payload...

    encryption_type = data[5]
    message_type = data[6]
    iv_length = data[7]

    print(f"  Encryption type: 0x{encryption_type:02X}")
    print(f"  Message type: 0x{message_type:02X}")
    print(f"  IV length: {iv_length}")

    iv = data[8 : 8 + iv_length]
    encrypted_payload = data[8 + iv_length :]

    print(f"  IV: {iv.hex()}")
    print(
        f"  Encrypted payload ({len(encrypted_payload)} bytes): {encrypted_payload.hex()}"
    )

    # Decrypt using AES-128-CBC
    cipher = AES.new(aes_key, AES.MODE_CBC, iv[:16])  # AES-128 uses 16-byte IV
    decrypted = cipher.decrypt(encrypted_payload)

    # Remove PKCS7 padding
    try:
        decrypted = unpad(decrypted, AES.block_size)
    except ValueError as e:
        print(f"  WARNING: Padding error: {e}")
        # Try without unpadding
        pass

    return decrypted


# We need to extract the AES key from Frame 13
# Frame 13 is the client's RSA-encrypted AES key
# But we need the server's private key to decrypt it...


# For now, let's analyze the structure without decryption
def analyze_packet_structure(hex_data: str, label: str):
    print(f"\n{'=' * 70}")
    print(f"{label}")
    print(f"{'=' * 70}")

    data = bytes.fromhex(hex_data)

    # Parse ProudNet header
    magic = data[0:2]
    # Length is encoded in a special way - 1 byte at offset 2
    length_byte = data[2]
    opcode = data[3]

    print(f"Magic: {magic.hex()} (should be 1357)")
    print(f"Length byte: 0x{length_byte:02X} ({length_byte})")
    print(f"Opcode: 0x{opcode:02X}")
    print(f"Total packet: {len(data)} bytes")
    print(f"\nRaw hex:\n{hex_data}")
    print(f"\nByte-by-byte breakdown:")
    for i in range(min(len(data), 48)):
        print(f"  [{i:2d}] 0x{data[i]:02X} ({data[i]:3d})", end="")
        if i < 2:
            print(" <- Magic" if i == 0 else "")
        elif i == 2:
            print(" <- Length byte")
        elif i == 3:
            print(f" <- Opcode (0x{opcode:02X})")
        elif i == 4:
            print(" <- Encryption type")
        elif i == 5:
            print(" <- Message type")
        elif i == 6:
            print(" <- IV length")
        elif i >= 7 and i < 39:
            print(f" <- IV[{i - 7}]")
        else:
            print(f" <- Data[{i - 39}]")

    if opcode == 0x25:
        encryption_type = data[4]
        message_type = data[5]
        iv_length = data[6]
        iv = data[7 : 7 + iv_length]
        encrypted_payload = data[7 + iv_length :]

        print(f"\n0x25 Encrypted Packet Structure:")
        print(f"  Encryption type: 0x{encryption_type:02X}")
        print(f"  Message type: 0x{message_type:02X}")
        print(f"  IV length: {iv_length}")
        print(f"  IV ({len(iv)} bytes): {iv.hex()}")
        print(f"  Encrypted payload ({len(encrypted_payload)} bytes):")

        if len(encrypted_payload) > 0:
            # Hex dump of encrypted payload
            for i in range(0, len(encrypted_payload), 16):
                chunk = encrypted_payload[i : i + 16]
                hex_str = " ".join(f"{b:02x}" for b in chunk)
                ascii_str = "".join(chr(b) if 32 <= b < 127 else "." for b in chunk)
                print(f"    {i:04x}  {hex_str:<48}  {ascii_str}")
        else:
            print("    (EMPTY - IV contains all data!)")


# Frame 18: Client → Server (0x0000 request)
frame18 = (
    "1357012425010120cad0485b90a0baa28eeceaac1fddc07427813f0a69515e4a7cd1bb5712f8846a"
)

# Frame 20: Server → Client (0x0000 response)
frame20 = (
    "1357012425010120d5fe7c371251bb19d1601b5aff2f2b8cc93cbcdd4918bcacb85fb264e0e7b2ab"
)

analyze_packet_structure(frame18, "Frame 18: Client → Server (0x0000 request)")
analyze_packet_structure(frame20, "Frame 20: Server → Client (0x0000 response)")

# Compare encrypted payloads
print(f"\n{'=' * 70}")
print("COMPARISON: Packets")
print(f"{'=' * 70}")

data18 = bytes.fromhex(frame18)
data20 = bytes.fromhex(frame20)

# The structure is:
# [0-1] Magic (0x1357)
# [2] Length byte
# [3] Opcode (0x25)
# [4] Encryption type
# [5] Message type
# [6] IV length
# [7-38] IV (32 bytes)
# [39+] Encrypted payload

print(f"\nFrame 18 (Client):")
print(f"  Length byte: 0x{data18[2]:02X}")
print(f"  Encryption: 0x{data18[4]:02X}")
print(f"  Message type: 0x{data18[5]:02X}")
print(f"  IV length: {data18[6]}")

print(f"\nFrame 20 (Server):")
print(f"  Length byte: 0x{data20[2]:02X}")
print(f"  Encryption: 0x{data20[4]:02X}")
print(f"  Message type: 0x{data20[5]:02X}")
print(f"  IV length: {data20[6]}")

# Compare IVs
iv18 = data18[7:39]
iv20 = data20[7:39]
print(f"\nFrame 18 IV: {iv18.hex()}")
print(f"Frame 20 IV: {iv20.hex()}")
print(f"Are IVs identical? {iv18 == iv20}")

# The actual encrypted payload (if any) starts at byte 39
payload18 = data18[39:]
payload20 = data20[39:]
print(f"\nFrame 18 encrypted payload: {len(payload18)} bytes")
if len(payload18) > 0:
    print(f"  {payload18.hex()}")
print(f"Frame 20 encrypted payload: {len(payload20)} bytes")
if len(payload20) > 0:
    print(f"  {payload20.hex()}")

print(f"\n{'=' * 70}")
print("KEY OBSERVATION:")
print(f"{'=' * 70}")
if len(payload18) == 0 and len(payload20) == 0:
    print("Both packets have ZERO encrypted payload bytes!")
    print("The 32-byte 'IV' field IS the encrypted data!")
    print("\nThis means:")
    print("  - Packet contains exactly 32 bytes of encrypted data")
    print("  - The '0x20' (32) at offset 6 is NOT the IV length")
    print("  - It might be: encrypted length, or part of the encryption scheme")
    print("\nWe need to decrypt these 32 bytes to see the 0x0000 packet contents!")
