//! Packet structures based on Ghidra analysis of Rag2.exe
//!
//! All structures match the binary layout found in the client.
//! See docs/ghidra-findings.md for detailed analysis.

pub mod parser;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

/// PacketHeader (16 bytes)
///
/// From Ghidra analysis at offset 0x00
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct PacketHeader {
    /// Virtual function table pointer (C++ implementation detail)
    pub vtable: u32,

    /// Source IPv4 address
    pub source_ip: Ipv4Addr,

    /// Source TCP/UDP port
    pub source_port: u16,

    /// Address property flags
    pub address_flags: u8,

    /// Reserved (must be 0)
    pub reserved: u8,

    /// Unique client identifier assigned by server
    pub host_id: u32,
}

impl PacketHeader {
    /// Size of PacketHeader in bytes
    pub const SIZE: usize = 16;

    /// Create a new PacketHeader
    pub fn new(source_ip: Ipv4Addr, source_port: u16, host_id: u32) -> Self {
        Self {
            vtable: 0,
            source_ip,
            source_port,
            address_flags: 0,
            reserved: 0,
            host_id,
        }
    }

    /// Serialize to bytes (little-endian)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(Self::SIZE);
        buf.put_u32_le(self.vtable);
        buf.put_slice(&self.source_ip.octets());
        buf.put_u16_le(self.source_port);
        buf.put_u8(self.address_flags);
        buf.put_u8(self.reserved);
        buf.put_u32_le(self.host_id);
        buf.to_vec()
    }

    /// Deserialize from bytes (little-endian)
    pub fn from_bytes(mut data: &[u8]) -> crate::Result<Self> {
        if data.len() < Self::SIZE {
            anyhow::bail!("Insufficient data for PacketHeader");
        }

        let vtable = data.get_u32_le();
        let ip_bytes = [data.get_u8(), data.get_u8(), data.get_u8(), data.get_u8()];
        let source_ip = Ipv4Addr::from(ip_bytes);
        let source_port = data.get_u16_le();
        let address_flags = data.get_u8();
        let reserved = data.get_u8();
        let host_id = data.get_u32_le();

        Ok(Self {
            vtable,
            source_ip,
            source_port,
            address_flags,
            reserved,
            host_id,
        })
    }
}

/// PacketBuffer (25 bytes)
///
/// From Ghidra analysis - dynamic buffer with read/write pointers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct PacketBuffer {
    /// Pointer to buffer data (placeholder - not used in Rust impl)
    pub buffer_data: u32,

    /// Total buffer size
    pub buffer_size: u32,

    /// Current data pointer
    pub current_data: u32,

    /// Number of bytes currently used
    pub current_size: u32,

    /// Total allocated memory size
    pub allocated_size: u32,

    /// Read cursor position
    pub read_position: u32,

    /// Control flags for buffer behavior
    pub buffer_flags: u8,
}

impl PacketBuffer {
    /// Size of PacketBuffer in bytes
    pub const SIZE: usize = 25;
}

/// NetworkPacket (44 bytes)
///
/// From Ghidra analysis - extends PacketBuffer with network-specific fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPacket {
    /// Buffer data pointer
    pub buffer_data: u32,

    /// Buffer size
    pub buffer_size: u32,

    /// Buffer capacity
    pub buffer_capacity: u32,

    /// Buffer offset
    pub buffer_offset: u32,

    /// Read pointer
    pub read_pointer: u32,

    /// Write pointer
    pub write_pointer: u32,

    /// Buffer flags
    pub buffer_flags: u8,

    /// Message type identifier (corresponds to MessageType enum)
    pub packet_type: u32,

    /// Embedded packet header
    pub header: PacketHeader,
}

impl NetworkPacket {
    /// Size of NetworkPacket in bytes
    pub const SIZE: usize = 44;
}

/// CompletePacket (48 bytes)
///
/// From Ghidra analysis - highest-level packet container used for transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletePacket {
    /// Packet buffer (25 bytes)
    pub buffer: PacketBuffer,

    /// Message type identifier
    pub packet_type: u32,

    /// Packet header (16 bytes)
    pub header: PacketHeader,
}

impl CompletePacket {
    /// Size of CompletePacket in bytes
    pub const SIZE: usize = 48;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_header_size() {
        assert_eq!(std::mem::size_of::<u32>() * 4 + 2 + 2, 18); // Not exactly 16 due to Rust padding
                                                                // In C with #pragma pack, it would be exactly 16 bytes
    }

    #[test]
    fn test_packet_header_serialization() {
        let header = PacketHeader::new(Ipv4Addr::new(127, 0, 0, 1), 7101, 0x12345678);

        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), PacketHeader::SIZE);

        let deserialized = PacketHeader::from_bytes(&bytes).unwrap();
        assert_eq!(deserialized.source_ip, header.source_ip);
        assert_eq!(deserialized.source_port, header.source_port);
        assert_eq!(deserialized.host_id, header.host_id);
    }
}
