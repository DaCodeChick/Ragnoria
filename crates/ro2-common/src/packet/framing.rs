//! RO2 Packet Framing Protocol
//!
//! All RO2 network packets use a consistent framing format:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │ Magic (u16 LE)  │ Size Byte │ Payload Size │ Payload   │
//! │ 0x5713          │ (u8)      │ (varint)     │ (bytes)   │
//! ├─────────────────┼───────────┼──────────────┼───────────┤
//! │ 2 bytes         │ 1 byte    │ 1/2/4 bytes  │ N bytes   │
//! └─────────────────────────────────────────────────────────┘
//! ```

use crate::Result;
use bytes::{Buf, BufMut};
use std::io::Cursor;

/// Magic number identifying RO2 packets (little endian)
pub const PACKET_MAGIC: u16 = 0x5713;

/// Magic bytes in network order (13 57 in hex dumps)
pub const PACKET_MAGIC_BYTES: [u8; 2] = [0x13, 0x57];

/// Minimum packet size (magic + size_byte + 1-byte varint + empty payload)
pub const MIN_PACKET_SIZE: usize = 4;

/// Maximum packet size (64KB - reasonable limit)
pub const MAX_PACKET_SIZE: usize = 65536;

/// RO2 packet frame structure
///
/// This represents the outer framing layer for all RO2 network packets.
/// The payload contains either ProudNet protocol messages (opcodes 0x01-0x32)
/// or encrypted game messages (opcode 0x25).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketFrame {
    /// Packet magic number (always 0x5713)
    pub magic: u16,

    /// Payload data (first byte is typically the opcode)
    pub payload: Vec<u8>,
}

impl PacketFrame {
    /// Create a new packet frame with payload
    pub fn new(payload: Vec<u8>) -> Self {
        Self {
            magic: PACKET_MAGIC,
            payload,
        }
    }

    /// Get the opcode (first byte of payload)
    pub fn opcode(&self) -> Option<u8> {
        self.payload.first().copied()
    }

    /// Get the opcode as u16 (first two bytes of payload, little endian)
    pub fn opcode_u16(&self) -> Option<u16> {
        if self.payload.len() >= 2 {
            Some(u16::from_le_bytes([self.payload[0], self.payload[1]]))
        } else {
            None
        }
    }

    /// Serialize the packet frame to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Write magic
        buf.put_u16_le(self.magic);

        // Write payload size as varint
        write_varint(&mut buf, self.payload.len() as u32);

        // Write payload
        buf.extend_from_slice(&self.payload);

        buf
    }

    /// Deserialize a packet frame from bytes
    ///
    /// Returns the packet frame and the number of bytes consumed.
    pub fn from_bytes(data: &[u8]) -> Result<(Self, usize)> {
        if data.len() < MIN_PACKET_SIZE {
            return Err(anyhow::anyhow!(
                "Packet too short: {} bytes (need at least {})",
                data.len(),
                MIN_PACKET_SIZE
            ));
        }

        let mut cursor = Cursor::new(data);

        // Read magic
        let magic = cursor.get_u16_le();
        if magic != PACKET_MAGIC {
            return Err(anyhow::anyhow!(
                "Invalid packet magic: 0x{:04x} (expected 0x{:04x})",
                magic,
                PACKET_MAGIC
            ));
        }

        // Read payload size (varint)
        let payload_size = read_varint(&mut cursor)? as usize;

        // Validate payload size
        if payload_size > MAX_PACKET_SIZE {
            return Err(anyhow::anyhow!(
                "Payload size too large: {} bytes (max {})",
                payload_size,
                MAX_PACKET_SIZE
            ));
        }

        let offset = cursor.position() as usize;

        // Check if we have enough data for payload
        if data.len() < offset + payload_size {
            return Err(anyhow::anyhow!(
                "Incomplete packet: need {} bytes, have {}",
                offset + payload_size,
                data.len()
            ));
        }

        // Extract payload
        let payload = data[offset..offset + payload_size].to_vec();

        let packet = Self { magic, payload };
        let total_size = offset + payload_size;

        Ok((packet, total_size))
    }

    /// Try to parse multiple packets from a buffer
    ///
    /// Returns all complete packets found and the number of bytes consumed.
    pub fn parse_multiple(data: &[u8]) -> Result<(Vec<Self>, usize)> {
        let mut packets = Vec::new();
        let mut offset = 0;

        while offset < data.len() {
            // Try to parse a packet
            match Self::from_bytes(&data[offset..]) {
                Ok((packet, size)) => {
                    packets.push(packet);
                    offset += size;
                }
                Err(_) => {
                    // Not enough data for another packet
                    break;
                }
            }
        }

        Ok((packets, offset))
    }
}

/// Write a variable-length integer
///
/// ProudNet varint format:
/// - 1 byte: size_byte (1, 2, or 4)
/// - N bytes: value (little endian)
pub fn write_varint(buf: &mut Vec<u8>, value: u32) {
    if value <= 0xFF {
        buf.put_u8(1); // Size byte
        buf.put_u8(value as u8);
    } else if value <= 0xFFFF {
        buf.put_u8(2); // Size byte
        buf.put_u16_le(value as u16);
    } else {
        buf.put_u8(4); // Size byte
        buf.put_u32_le(value);
    }
}

/// Read a variable-length integer
///
/// ProudNet varint format:
/// - 1 byte: size_byte (1, 2, or 4)
/// - N bytes: value (little endian)
pub fn read_varint(cursor: &mut Cursor<&[u8]>) -> Result<u32> {
    if !cursor.has_remaining() {
        return Err(anyhow::anyhow!("No data for varint size byte"));
    }

    let size_byte = cursor.get_u8();

    match size_byte {
        1 => {
            if !cursor.has_remaining() {
                return Err(anyhow::anyhow!("Not enough data for 1-byte varint"));
            }
            Ok(cursor.get_u8() as u32)
        }
        2 => {
            if cursor.remaining() < 2 {
                return Err(anyhow::anyhow!("Not enough data for 2-byte varint"));
            }
            Ok(cursor.get_u16_le() as u32)
        }
        4 => {
            if cursor.remaining() < 4 {
                return Err(anyhow::anyhow!("Not enough data for 4-byte varint"));
            }
            Ok(cursor.get_u32_le())
        }
        _ => Err(anyhow::anyhow!("Invalid varint size byte: {}", size_byte)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_roundtrip() {
        let test_values = vec![0, 1, 127, 255, 256, 65535, 65536, 0xFFFFFFFF];

        for value in test_values {
            let mut buf = Vec::new();
            write_varint(&mut buf, value);

            let mut cursor = Cursor::new(buf.as_slice());
            let parsed = read_varint(&mut cursor).unwrap();

            assert_eq!(parsed, value, "Failed for value {}", value);
        }
    }

    #[test]
    fn test_packet_frame_parsing() {
        // Example policy request packet: 13 57 01 05 2f 0f 00 00 40
        let data = hex::decode("135701052f0f000040").unwrap();

        let (packet, size) = PacketFrame::from_bytes(&data).unwrap();

        assert_eq!(packet.magic, PACKET_MAGIC);
        assert_eq!(packet.payload.len(), 5);
        assert_eq!(packet.payload, vec![0x2f, 0x0f, 0x00, 0x00, 0x40]);
        assert_eq!(packet.opcode(), Some(0x2f));
        assert_eq!(size, 9);
    }

    #[test]
    fn test_packet_frame_roundtrip() {
        let payload = vec![0x25, 0x01, 0x02, 0x03, 0x04];
        let packet = PacketFrame::new(payload.clone());

        let bytes = packet.to_bytes();
        let (parsed, size) = PacketFrame::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.magic, PACKET_MAGIC);
        assert_eq!(parsed.payload, payload);
        assert_eq!(size, bytes.len());
    }

    #[test]
    fn test_packet_frame_invalid_magic() {
        let data = hex::decode("FFFF01050102030405").unwrap();
        let result = PacketFrame::from_bytes(&data);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid packet magic")
        );
    }

    #[test]
    fn test_packet_frame_incomplete() {
        // Magic + size byte + varint = 4 bytes, but payload claims 100 bytes
        let data = hex::decode("13570164").unwrap(); // Claims 100 bytes but none present
        let result = PacketFrame::from_bytes(&data);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Incomplete packet")
        );
    }

    #[test]
    fn test_parse_multiple_packets() {
        // Two packets: [13 57 01 03 AA BB CC] [13 57 01 02 DD EE]
        let data = hex::decode("13570103AABBCC13570102DDEE").unwrap();

        let (packets, consumed) = PacketFrame::parse_multiple(&data).unwrap();

        assert_eq!(packets.len(), 2);
        assert_eq!(packets[0].payload, vec![0xAA, 0xBB, 0xCC]);
        assert_eq!(packets[1].payload, vec![0xDD, 0xEE]);
        assert_eq!(consumed, data.len());
    }

    #[test]
    fn test_opcode_extraction() {
        let packet = PacketFrame::new(vec![0x25, 0x01, 0x02]);

        assert_eq!(packet.opcode(), Some(0x25));
        assert_eq!(packet.opcode_u16(), Some(0x0125));
    }
}
