#!/usr/bin/env python3
"""
Calculate file offset from virtual address in a PE file.
"""

import struct
import sys


def read_pe_offset(pe_path, virtual_address):
    """Calculate file offset from virtual address."""
    with open(pe_path, "rb") as f:
        # Read DOS header
        f.seek(0)
        dos_header = f.read(64)
        if dos_header[:2] != b"MZ":
            print("Error: Not a valid PE file (missing MZ signature)")
            return None

        # Get PE header offset
        pe_offset = struct.unpack("<I", dos_header[0x3C:0x40])[0]

        # Read PE signature
        f.seek(pe_offset)
        pe_sig = f.read(4)
        if pe_sig != b"PE\x00\x00":
            print("Error: Not a valid PE file (missing PE signature)")
            return None

        # Read COFF header
        coff_header = f.read(20)
        num_sections = struct.unpack("<H", coff_header[2:4])[0]
        optional_header_size = struct.unpack("<H", coff_header[16:18])[0]

        # Read Optional header to get ImageBase
        optional_header = f.read(optional_header_size)
        image_base = struct.unpack("<I", optional_header[28:32])[0]

        print(f"Image Base: 0x{image_base:08X}")
        print(f"Virtual Address: 0x{virtual_address:08X}")
        print(f"Number of sections: {num_sections}")
        print()

        # Calculate RVA
        rva = virtual_address - image_base
        print(f"RVA: 0x{rva:08X}")
        print()

        # Read section headers
        sections = []
        for i in range(num_sections):
            section_header = f.read(40)
            name = section_header[:8].rstrip(b"\x00").decode("ascii", errors="ignore")
            virtual_size = struct.unpack("<I", section_header[8:12])[0]
            virtual_address_sec = struct.unpack("<I", section_header[12:16])[0]
            raw_size = struct.unpack("<I", section_header[16:20])[0]
            raw_offset = struct.unpack("<I", section_header[20:24])[0]

            sections.append(
                {
                    "name": name,
                    "virtual_address": virtual_address_sec,
                    "virtual_size": virtual_size,
                    "raw_offset": raw_offset,
                    "raw_size": raw_size,
                }
            )

            print(
                f"Section: {name:8s} VA: 0x{virtual_address_sec:08X} VSize: 0x{virtual_size:08X} "
                f"Raw: 0x{raw_offset:08X} RSize: 0x{raw_size:08X}"
            )

        print()

        # Find which section contains our RVA
        for section in sections:
            va_start = section["virtual_address"]
            va_end = va_start + section["virtual_size"]

            if va_start <= rva < va_end:
                # Calculate file offset
                offset_in_section = rva - va_start
                file_offset = section["raw_offset"] + offset_in_section

                print(f"✓ Found in section: {section['name']}")
                print(f"  Section VA range: 0x{va_start:08X} - 0x{va_end:08X}")
                print(f"  Offset in section: 0x{offset_in_section:08X}")
                print(f"  FILE OFFSET: 0x{file_offset:08X}")

                return file_offset

        print("Error: RVA not found in any section")
        return None


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: find_pe_offset.py <pe_file> <virtual_address_hex>")
        print("Example: find_pe_offset.py Rag2.exe 0x00A4FFA0")
        sys.exit(1)

    pe_path = sys.argv[1]
    va = int(sys.argv[2], 16)

    print(f"Analyzing: {pe_path}")
    print(f"Target VA: 0x{va:08X}")
    print("=" * 70)
    print()

    offset = read_pe_offset(pe_path, va)

    if offset:
        print()
        print("=" * 70)
        print(f"RESULT: Virtual Address 0x{va:08X} = File Offset 0x{offset:08X}")
