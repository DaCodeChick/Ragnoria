#!/usr/bin/env python3
"""
Compare our server's 0x0000 response with the official server's response from PCAP
"""

import sys

# Official server response from PCAP (Frame 20)
OFFICIAL_RESPONSE = (
    "1357012425010120d5fe7c371251bb19d1601b5aff2f2b8cc93cbcdd4918bcacb85fb264e0e7b2ab"
)


def parse_packet(hex_str):
    """Parse ProudNet encrypted packet structure"""
    data = bytes.fromhex(hex_str)

    result = {
        "total_len": len(data),
        "magic": data[0:2].hex(),
        "varint_size": data[2],
        "payload_len": data[3],
        "opcode": data[4],
        "enc_type": data[5],
        "msg_type": data[6],
        "enc_data_len": data[7],
        "encrypted": data[8:].hex(),
        "encrypted_bytes": len(data[8:]),
    }

    return result


def compare_packets(our_hex, official_hex):
    """Compare our packet with official server's packet"""
    our = parse_packet(our_hex)
    official = parse_packet(official_hex)

    print("=" * 70)
    print("PACKET COMPARISON")
    print("=" * 70)

    print(f"\n{'Field':<20} {'Our Server':<25} {'Official':<25} {'Match':<5}")
    print("-" * 70)

    def compare_field(name, our_val, off_val):
        match = "✓" if our_val == off_val else "✗"
        print(f"{name:<20} {str(our_val):<25} {str(off_val):<25} {match:<5}")
        return our_val == off_val

    all_match = True
    all_match &= compare_field("Total length", our["total_len"], official["total_len"])
    all_match &= compare_field("Magic", our["magic"], official["magic"])
    all_match &= compare_field(
        "Varint size", our["varint_size"], official["varint_size"]
    )
    all_match &= compare_field(
        "Payload length", our["payload_len"], official["payload_len"]
    )
    all_match &= compare_field(
        "Opcode", f"0x{our['opcode']:02x}", f"0x{official['opcode']:02x}"
    )
    all_match &= compare_field(
        "Encryption type", f"0x{our['enc_type']:02x}", f"0x{official['enc_type']:02x}"
    )
    all_match &= compare_field(
        "Message type", f"0x{our['msg_type']:02x}", f"0x{official['msg_type']:02x}"
    )
    all_match &= compare_field(
        "Enc data length", our["enc_data_len"], official["enc_data_len"]
    )
    all_match &= compare_field(
        "Encrypted bytes", our["encrypted_bytes"], official["encrypted_bytes"]
    )

    print(f"\nEncrypted data comparison:")
    print(f"  Our:      {our['encrypted']}")
    print(f"  Official: {official['encrypted']}")
    print(
        f"  Match:    {'✓ IDENTICAL' if our['encrypted'] == official['encrypted'] else '✗ DIFFERENT (expected - different AES keys)'}"
    )

    print(f"\n{'=' * 70}")
    if all_match and our["encrypted"] != official["encrypted"]:
        print("✓ STRUCTURE MATCHES PERFECTLY!")
        print("✓ Encrypted data is different (expected - different session keys)")
        print("✓ Our packet format is CORRECT!")
        print("\nCONCLUSION: The problem is NOT in the packet structure.")
        print(
            "            The problem is in the DECRYPTED CONTENT of the 0x0000 response."
        )
        print("            We need to test different values for suspicious bytes:")
        print("            - Bytes 0x16-0x19 (currently 0x803F0000)")
        print("            - Bytes 0x12-0x15 (currently 0x07022500)")
    elif not all_match:
        print("✗ STRUCTURE MISMATCH DETECTED!")
        print(
            "\nOur packet structure is WRONG. Fix the structure first before testing content."
        )
    else:
        print("⚠️ UNEXPECTED: Both structure AND encrypted data match")
        print("   This should not happen unless we're using the same AES key...")
    print("=" * 70)


if __name__ == "__main__":
    print("=" * 70)
    print("RO2 Server 0x0000 Response Comparison Tool")
    print("=" * 70)
    print(f"\nOfficial server packet (from PCAP Frame 20):")
    print(f"  {OFFICIAL_RESPONSE}\n")

    if len(sys.argv) > 1:
        our_packet = sys.argv[1]
        print(f"Our server packet (from command line):")
        print(f"  {our_packet}\n")
        compare_packets(our_packet, OFFICIAL_RESPONSE)
    else:
        print("Usage:")
        print(f"  python3 {sys.argv[0]} <hex_packet_from_our_server>")
        print("\nTo get our server's packet:")
        print("  1. Run ./target/release/ro2-login")
        print("  2. Connect with RO2 client")
        print("  3. Look for log line: 'Full hex: ...'")
        print("  4. Copy the hex string and run:")
        print(f"     python3 {sys.argv[0]} <hex_string>")

        print("\n" + "=" * 70)
        print("REFERENCE: Official server packet structure")
        print("=" * 70)
        official = parse_packet(OFFICIAL_RESPONSE)
        for key, val in official.items():
            if key != "encrypted":
                print(f"  {key}: {val}")
        print(
            f"  encrypted: {official['encrypted'][:32]}... ({official['encrypted_bytes']} bytes)"
        )
