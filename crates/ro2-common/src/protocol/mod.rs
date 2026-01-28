//! ProudNet RMI protocol implementation

use serde::{Deserialize, Serialize};

/// ProudNet RMI message types
///
/// Based on Ghidra analysis of Rag2.exe. Message IDs are placeholders
/// and must be determined through packet capture analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum MessageType {
    // ========== Authentication & Login ==========
    ReqLogin = 0x0001,
    AnsLogin = 0x0002,
    ReqLoginChannel = 0x0003,
    AnsLoginChannel = 0x0004,
    ReqServerStatus = 0x0005,
    AckServerStatus = 0x0006,
    AckVersionCheck = 0x0007,
    ReqPing = 0x0008,
    
    // Notifications
    NfyServerTime = 0x1000,
    NfyServerTimeToLoginPC = 0x1001,
    NfyChannelDisconnect = 0x1002,
    
    // Placeholder for unknown messages
    Unknown = 0xFFFFFFFF,
}

impl MessageType {
    /// Convert u32 to MessageType
    pub fn from_u32(value: u32) -> Self {
        match value {
            0x0001 => Self::ReqLogin,
            0x0002 => Self::AnsLogin,
            0x0003 => Self::ReqLoginChannel,
            0x0004 => Self::AnsLoginChannel,
            0x0005 => Self::ReqServerStatus,
            0x0006 => Self::AckServerStatus,
            0x0007 => Self::AckVersionCheck,
            0x0008 => Self::ReqPing,
            0x1000 => Self::NfyServerTime,
            0x1001 => Self::NfyServerTimeToLoginPC,
            0x1002 => Self::NfyChannelDisconnect,
            _ => Self::Unknown,
        }
    }
    
    /// Convert MessageType to u32
    pub fn to_u32(self) -> u32 {
        self as u32
    }
}

/// Trait for ProudNet packet serialization
pub trait ProudNetPacket: Sized {
    /// Serialize packet to bytes
    fn serialize(&self) -> crate::Result<Vec<u8>>;
    
    /// Deserialize packet from bytes
    fn deserialize(data: &[u8]) -> crate::Result<Self>;
}

pub mod rmi;
