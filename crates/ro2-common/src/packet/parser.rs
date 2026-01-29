//! ProudNet RMI packet parser
//!
//! Implements parsing logic for incoming network packets based on the
//! ProudNet protocol structure discovered through Ghidra analysis.

use crate::protocol::MessageType;
use bytes::{Buf, Bytes};

/// Parsed ProudNet RMI message
#[derive(Debug, Clone)]
pub struct RmiMessage {
    /// Packet signature/magic bytes
    pub magic: u32,

    /// Total packet length (excluding magic and length fields)
    pub length: u32,

    /// Message type identifier
    pub message_id: u16,

    /// Protocol flags or version
    pub flags: u16,

    /// Packet sequence number
    pub sequence: u32,

    /// Message payload data
    pub payload: Bytes,
}

impl RmiMessage {
    /// Minimum header size for an RMI message
    pub const HEADER_SIZE: usize = 16;

    /// Parse an RMI message from raw bytes
    ///
    /// Expected structure:
    /// ```text
    /// Offset | Size | Field
    /// -------|------|----------
    /// 0x00   | 4    | magic
    /// 0x04   | 4    | length
    /// 0x08   | 2    | message_id
    /// 0x0A   | 2    | flags
    /// 0x0C   | 4    | sequence
    /// 0x10   | N    | payload
    /// ```
    pub fn parse(data: &[u8]) -> crate::Result<Self> {
        if data.len() < Self::HEADER_SIZE {
            anyhow::bail!(
                "Packet too short: expected at least {} bytes, got {}",
                Self::HEADER_SIZE,
                data.len()
            );
        }

        let mut buf = Bytes::copy_from_slice(data);

        let magic = buf.get_u32_le();
        let length = buf.get_u32_le();
        let message_id = buf.get_u16_le();
        let flags = buf.get_u16_le();
        let sequence = buf.get_u32_le();

        // Validate length
        let expected_total = Self::HEADER_SIZE as u32 + length;
        if data.len() < expected_total as usize {
            anyhow::bail!(
                "Incomplete packet: header claims {} bytes, but only {} available",
                expected_total,
                data.len()
            );
        }

        // Extract payload
        let payload = buf.slice(..length as usize);

        Ok(Self {
            magic,
            length,
            message_id,
            flags,
            sequence,
            payload,
        })
    }

    /// Get the message type enum value (if known)
    pub fn message_type(&self) -> Option<MessageType> {
        MessageType::from_id(self.message_id)
    }

    /// Check if this message is encrypted (heuristic based on magic)
    pub fn is_encrypted(&self) -> bool {
        // TODO: Determine actual ProudNet encryption magic values
        // Common patterns: 0x5A5A5A5A (encrypted), 0x50524F55 ('PROU' plaintext)
        self.magic != 0x50524F55 // 'PROU' in little-endian
    }

    /// Serialize back to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::HEADER_SIZE + self.payload.len());

        bytes.extend_from_slice(&self.magic.to_le_bytes());
        bytes.extend_from_slice(&self.length.to_le_bytes());
        bytes.extend_from_slice(&self.message_id.to_le_bytes());
        bytes.extend_from_slice(&self.flags.to_le_bytes());
        bytes.extend_from_slice(&self.sequence.to_le_bytes());
        bytes.extend_from_slice(&self.payload);

        bytes
    }
}

/// Build an RMI response message
pub struct RmiMessageBuilder {
    message_id: u16,
    sequence: u32,
    payload: Vec<u8>,
}

impl RmiMessageBuilder {
    /// Create a new message builder
    pub fn new(message_id: u16, sequence: u32) -> Self {
        Self {
            message_id,
            sequence,
            payload: Vec::new(),
        }
    }

    /// Add payload data
    pub fn payload(mut self, data: &[u8]) -> Self {
        self.payload.extend_from_slice(data);
        self
    }

    /// Write a string to payload (length-prefixed)
    pub fn write_string(mut self, s: &str) -> Self {
        let len = s.len() as u32;
        self.payload.extend_from_slice(&len.to_le_bytes());
        self.payload.extend_from_slice(s.as_bytes());
        self
    }

    /// Write a u32 to payload
    pub fn write_u32(mut self, value: u32) -> Self {
        self.payload.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Write a u16 to payload
    pub fn write_u16(mut self, value: u16) -> Self {
        self.payload.extend_from_slice(&value.to_le_bytes());
        self
    }

    /// Write a u8 to payload
    pub fn write_u8(mut self, value: u8) -> Self {
        self.payload.push(value);
        self
    }

    /// Build the final RmiMessage
    pub fn build(self) -> RmiMessage {
        RmiMessage {
            magic: 0x50524F55, // 'PROU' - plaintext for now
            length: self.payload.len() as u32,
            message_id: self.message_id,
            flags: 0x0001, // Default flags
            sequence: self.sequence,
            payload: Bytes::from(self.payload),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rmi_message() {
        // Construct a test packet
        let mut data = Vec::new();
        data.extend_from_slice(&0x50524F55u32.to_le_bytes()); // 'PROU'
        data.extend_from_slice(&8u32.to_le_bytes()); // length = 8 bytes payload
        data.extend_from_slice(&0x0123u16.to_le_bytes()); // message_id
        data.extend_from_slice(&0x0001u16.to_le_bytes()); // flags
        data.extend_from_slice(&42u32.to_le_bytes()); // sequence
        data.extend_from_slice(b"testdata"); // 8 bytes payload

        let msg = RmiMessage::parse(&data).unwrap();

        assert_eq!(msg.magic, 0x50524F55);
        assert_eq!(msg.length, 8);
        assert_eq!(msg.message_id, 0x0123);
        assert_eq!(msg.flags, 0x0001);
        assert_eq!(msg.sequence, 42);
        assert_eq!(&msg.payload[..], b"testdata");
    }

    #[test]
    fn test_rmi_message_builder() {
        let msg = RmiMessageBuilder::new(0x0124, 1)
            .write_string("admin")
            .write_u32(0x12345678)
            .build();

        assert_eq!(msg.message_id, 0x0124);
        assert_eq!(msg.sequence, 1);

        // Parse it back
        let bytes = msg.to_bytes();
        let parsed = RmiMessage::parse(&bytes).unwrap();

        assert_eq!(parsed.message_id, msg.message_id);
        assert_eq!(parsed.sequence, msg.sequence);
    }

    #[test]
    fn test_insufficient_data() {
        let data = vec![0u8; 8]; // Less than HEADER_SIZE
        let result = RmiMessage::parse(&data);
        assert!(result.is_err());
    }
}
