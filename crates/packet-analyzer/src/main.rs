use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "packet-analyzer")]
#[command(about = "Analyze RO2 ProudNet packet captures", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a Wireshark hex dump file
    File {
        /// Path to hex dump file
        path: PathBuf,
    },
    /// Parse a hex string directly
    Hex {
        /// Hex string (e.g., "504F5255...")
        #[arg(short, long)]
        data: String,
    },
    /// Interactive mode - paste hex and analyze
    Interactive,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::File { path } => {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read file: {:?}", path))?;
            analyze_hex_dump(&content)?;
        }
        Commands::Hex { data } => {
            let bytes = parse_hex_string(&data)?;
            analyze_packet(&bytes)?;
        }
        Commands::Interactive => {
            interactive_mode()?;
        }
    }

    Ok(())
}

fn analyze_hex_dump(content: &str) -> Result<()> {
    println!("=== Analyzing Hex Dump ===\n");

    // Try to extract hex data from Wireshark format
    let mut all_bytes = Vec::new();

    for line in content.lines() {
        // Wireshark format: "0000  50 52 4f 55 ..."
        if let Some(hex_part) = extract_hex_from_line(line) {
            let bytes = parse_hex_string(&hex_part)?;
            all_bytes.extend(bytes);
        }
    }

    if all_bytes.is_empty() {
        println!("No hex data found in file. Make sure it's a Wireshark hex dump.");
        return Ok(());
    }

    println!("Total bytes extracted: {}\n", all_bytes.len());
    analyze_packet(&all_bytes)?;

    Ok(())
}

fn extract_hex_from_line(line: &str) -> Option<String> {
    // Match Wireshark format: "0000  50 52 4f 55 ..."
    // Skip lines that don't start with hex offset
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.is_empty() {
        return None;
    }

    // First part should be offset (4-8 hex digits)
    if !parts[0].chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }

    // Collect remaining hex bytes (before ASCII part if present)
    let mut hex_string = String::new();
    for part in &parts[1..] {
        // Stop at ASCII representation (usually after a bunch of hex pairs)
        if part.len() > 2 && !part.chars().all(|c| c.is_ascii_hexdigit()) {
            break;
        }
        hex_string.push_str(part);
    }

    if hex_string.is_empty() {
        None
    } else {
        Some(hex_string)
    }
}

fn parse_hex_string(hex: &str) -> Result<Vec<u8>> {
    let clean = hex.replace(" ", "").replace("\n", "").replace("\r", "");
    hex::decode(&clean).context("Invalid hex string")
}

fn analyze_packet(bytes: &[u8]) -> Result<()> {
    if bytes.len() < 16 {
        println!(
            "⚠️  Packet too short ({} bytes). Minimum size is 16 bytes for packet header.",
            bytes.len()
        );
        return Ok(());
    }

    println!("=== Packet Analysis ===\n");

    // Display raw hex dump
    println!("Raw Hex Dump:");
    print_hex_dump(bytes);
    println!();

    // Try to parse as ProudNet packet
    println!("=== ProudNet Packet Structure ===\n");

    // Check for ProudNet magic (unknown, but we can look for patterns)
    let magic = &bytes[0..4];
    println!(
        "Magic/Signature:  {:02X} {:02X} {:02X} {:02X}",
        magic[0], magic[1], magic[2], magic[3]
    );

    if magic == b"PROU" {
        println!("                  ✓ Recognized ProudNet signature");
    } else {
        println!("                  ⚠️  Unknown signature (expected 'PROU' or similar)");
    }
    println!();

    // Packet length
    let packet_length = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    println!(
        "Packet Length:    {} bytes (0x{:04X})",
        packet_length, packet_length
    );

    if packet_length as usize == bytes.len() - 8 {
        println!("                  ✓ Length matches remaining data");
    } else {
        println!(
            "                  ⚠️  Length mismatch (expected {} bytes after header)",
            packet_length
        );
    }
    println!();

    // Message ID (THIS IS CRITICAL)
    let message_id = u16::from_le_bytes([bytes[8], bytes[9]]);
    println!("Message ID:       0x{:04X} ({})", message_id, message_id);
    println!("                  ⚠️  UPDATE MessageType ENUM WITH THIS VALUE");
    println!();

    // Flags/version
    let flags = u16::from_le_bytes([bytes[10], bytes[11]]);
    println!("Flags/Version:    0x{:04X}", flags);
    println!();

    // Sequence number
    let sequence = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
    println!("Sequence Number:  {}", sequence);
    println!();

    // Payload
    if bytes.len() > 16 {
        let payload = &bytes[16..];
        println!("=== Payload ({} bytes) ===\n", payload.len());
        print_hex_dump(payload);
        println!();

        // Try to identify common patterns
        analyze_payload(payload, message_id);
    } else {
        println!("No payload data.");
    }

    println!("\n=== Suggested Code Update ===\n");
    println!("Add to crates/ro2-common/src/protocol/mod.rs:");
    println!();
    println!(
        "    UnknownMessage_{:04X} = 0x{:04X},",
        message_id, message_id
    );
    println!();
    println!("Then rename based on packet direction and content analysis.");

    Ok(())
}

fn print_hex_dump(bytes: &[u8]) {
    for (i, chunk) in bytes.chunks(16).enumerate() {
        print!("{:04X}  ", i * 16);

        // Hex bytes
        for (j, byte) in chunk.iter().enumerate() {
            print!("{:02X} ", byte);
            if j == 7 {
                print!(" ");
            }
        }

        // Padding if last line is short
        if chunk.len() < 16 {
            for j in chunk.len()..16 {
                print!("   ");
                if j == 7 {
                    print!(" ");
                }
            }
        }

        print!(" ");

        // ASCII representation
        for byte in chunk {
            let c = if byte.is_ascii_graphic() || *byte == b' ' {
                *byte as char
            } else {
                '.'
            };
            print!("{}", c);
        }

        println!();
    }
}

fn analyze_payload(payload: &[u8], message_id: u16) {
    println!("=== Payload Pattern Analysis ===\n");

    // Check for null-terminated strings
    let mut potential_strings = Vec::new();
    let mut current_string = Vec::new();

    for &byte in payload {
        if byte == 0 {
            if !current_string.is_empty() {
                if let Ok(s) = String::from_utf8(current_string.clone()) {
                    if s.len() >= 3 && s.chars().all(|c| c.is_ascii_graphic() || c.is_whitespace())
                    {
                        potential_strings.push(s);
                    }
                }
                current_string.clear();
            }
        } else if byte.is_ascii_graphic() || byte == b' ' {
            current_string.push(byte);
        } else {
            current_string.clear();
        }
    }

    if !potential_strings.is_empty() {
        println!("Potential strings found:");
        for s in &potential_strings {
            println!("  - \"{}\"", s);
        }
        println!();
    }

    // Check for length-prefixed strings (common in ProudNet)
    if payload.len() >= 4 {
        let len1 = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
        if len1 > 0 && len1 < 256 && (4 + len1 as usize) <= payload.len() {
            if let Ok(s) = String::from_utf8(payload[4..(4 + len1 as usize)].to_vec()) {
                if s.chars().all(|c| c.is_ascii_graphic() || c.is_whitespace()) {
                    println!("Length-prefixed string detected at offset 0:");
                    println!("  Length: {} bytes", len1);
                    println!("  String: \"{}\"", s);
                    println!();
                }
            }
        }
    }

    // Guess message type based on ID range
    let message_type = match message_id {
        0x0000..=0x00FF => "Likely system/control message",
        0x0100..=0x01FF => "Likely authentication/login message",
        0x0200..=0x02FF => "Likely lobby/channel message",
        0x0300..=0x03FF => "Likely character management message",
        0x0400..=0x0FFF => "Likely gameplay message",
        _ => "Unknown category",
    };

    println!("Message category guess: {}", message_type);
    println!();

    // Entropy check (high entropy = likely encrypted)
    let entropy = calculate_entropy(payload);
    println!("Payload entropy: {:.2} bits/byte", entropy);
    if entropy > 7.5 {
        println!("  ⚠️  High entropy detected - payload may be encrypted");
    } else if entropy < 4.0 {
        println!("  ✓ Low entropy - payload likely plaintext or structured data");
    } else {
        println!("  ~ Medium entropy - mixed content or compressed data");
    }
}

fn calculate_entropy(data: &[u8]) -> f64 {
    let mut counts = [0u32; 256];
    for &byte in data {
        counts[byte as usize] += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0;

    for &count in &counts {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }

    entropy
}

fn interactive_mode() -> Result<()> {
    println!("=== Interactive Packet Analyzer ===");
    println!("Paste hex data (Ctrl+D or Ctrl+Z to finish):\n");

    use std::io::{self, Read};
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    let bytes = parse_hex_string(&buffer)?;
    analyze_packet(&bytes)?;

    Ok(())
}
