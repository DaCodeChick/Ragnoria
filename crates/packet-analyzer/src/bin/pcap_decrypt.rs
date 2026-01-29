//! PCAP analyzer for RO2 login sequence
//!
//! This tool parses the ro2login.pcapng file and attempts to decrypt
//! the 0x25 encrypted packets to extract game message opcodes.

use ro2_common::crypto::ProudNetCrypto;
use ro2_common::packet::PacketFrame;
use std::fs;

fn main() -> anyhow::Result<()> {
    println!("RO2 Login PCAP Analyzer");
    println!("=======================\n");

    // Read packet data from tshark export
    let packets_file = "/tmp/packets.txt";

    if !std::path::Path::new(packets_file).exists() {
        eprintln!("Error: {} not found", packets_file);
        eprintln!(
            "Run: tshark -r /home/admin/Downloads/ro2login.pcapng -Y 'tcp.port == 7101 && tcp.len > 0' -T fields -e frame.number -e tcp.srcport -e data > /tmp/packets.txt"
        );
        return Ok(());
    }

    let data = fs::read_to_string(packets_file)?;

    // Parse all packets
    println!("Parsing packets from capture...\n");

    let mut crypto = ProudNetCrypto::new();
    let mut rsa_key_found = false;
    let session_key_found = false;

    for line in data.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            continue;
        }

        let frame: u32 = parts[0].parse().unwrap_or(0);
        let src_port: u16 = parts[1].parse().unwrap_or(0);
        let hex_data = parts[2];

        let direction = if src_port == 63148 { "C->S" } else { "S->C" };

        // Decode hex
        let raw_data = match hex::decode(hex_data) {
            Ok(d) => d,
            Err(_) => continue,
        };

        // Parse packet frame
        let (packet, _) = match PacketFrame::from_bytes(&raw_data) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let opcode = packet.opcode().unwrap_or(0);

        // Look for key packets
        match opcode {
            0x04 if !rsa_key_found => {
                println!("Frame {} [{}] - RSA Public Key (0x04)", frame, direction);
                println!("  Payload size: {} bytes", packet.payload.len());

                // Try to find RSA key in payload
                // From analysis, key starts at offset 0x30 (48 bytes into payload)
                let key_offset = 43; // Offset in opcode-stripped payload, or 48 in full payload

                if packet.payload.len() > key_offset + 140 {
                    let key_data = &packet.payload[key_offset..];

                    // Look for ASN.1 DER header (30 81 89 or 30 82 ...)
                    if key_data[0] == 0x30 {
                        println!("  Found ASN.1 DER structure at offset {}", key_offset);

                        // Try to parse up to 200 bytes as potential RSA key
                        let potential_key = &key_data[..200.min(key_data.len())];

                        match crypto.set_rsa_public_key_from_der(potential_key) {
                            Ok(_) => {
                                println!("  ✓ Successfully parsed RSA public key!");
                                rsa_key_found = true;
                            }
                            Err(e) => {
                                println!("  ✗ Failed to parse RSA key: {}", e);
                                println!(
                                    "     First bytes: {}",
                                    hex::encode(&key_data[..20.min(key_data.len())])
                                );
                            }
                        }
                    }
                }
                println!();
            }

            0x05 if rsa_key_found && !session_key_found => {
                println!(
                    "Frame {} [{}] - Encrypted Session Key (0x05)",
                    frame, direction
                );
                println!("  Payload size: {} bytes", packet.payload.len());

                // Skip opcode and extract encrypted key
                if packet.payload.len() > 4 {
                    let encrypted_key = &packet.payload[4..];
                    println!("  Encrypted key size: {} bytes", encrypted_key.len());

                    // Note: We can't decrypt this without the server's private key
                    println!("  ⚠ Cannot decrypt without server's RSA private key");
                    println!("     (Would need to extract from server executable)");
                }
                println!();
            }

            0x25 => {
                println!("Frame {} [{}] - Encrypted Packet (0x25)", frame, direction);
                println!("  Payload size: {} bytes", packet.payload.len());

                if packet.payload.len() > 1 {
                    let sub_opcode = packet.payload[1];
                    println!("  Sub-opcode: 0x{:02x}", sub_opcode);
                }

                if session_key_found {
                    // Try to decrypt
                    match crypto.decrypt_packet_0x25(&packet.payload) {
                        Ok(decrypted) => {
                            println!("  ✓ Decrypted! {} bytes", decrypted.len());

                            // Try to parse as game message
                            if decrypted.len() >= 2 {
                                let game_opcode = u16::from_le_bytes([decrypted[0], decrypted[1]]);
                                println!("  Game opcode: 0x{:04x}", game_opcode);
                                println!(
                                    "  Data: {}",
                                    hex::encode(&decrypted[..32.min(decrypted.len())])
                                );
                            }
                        }
                        Err(e) => {
                            println!("  ✗ Decryption failed: {}", e);
                        }
                    }
                } else {
                    println!("  ⚠ Cannot decrypt: No session key available");
                }
                println!();
            }

            _ => {}
        }
    }

    println!("\n=================================================================");
    println!("Analysis Summary:");
    println!("=================================================================");
    println!("RSA Public Key Found: {}", rsa_key_found);
    println!("Session Key Decrypted: {}", session_key_found);
    println!();

    if !session_key_found {
        println!("⚠ LIMITATION:");
        println!("We can parse the RSA public key from the server,");
        println!("but we cannot decrypt the client's session key (0x05)");
        println!("without the server's RSA private key.");
        println!();
        println!("To decrypt 0x25 packets, we need either:");
        println!("1. Extract RSA private key from server executable");
        println!("2. Perform MITM with custom client that logs session key");
        println!("3. Reverse engineer AES key derivation from Ghidra");
    }

    Ok(())
}
