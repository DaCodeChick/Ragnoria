//! RO2 Common Library
//!
//! Shared functionality for Ragnarok Online 2 server emulator including:
//! - Protocol definitions (ProudNet RMI)
//! - Packet structures
//! - Cryptography (AES/RSA)
//! - Database models

pub mod protocol;
pub mod packet;
pub mod crypto;
pub mod database;

pub use packet::{PacketHeader, PacketBuffer, NetworkPacket};
pub use protocol::MessageType;

/// Common result type for RO2 operations
pub type Result<T> = anyhow::Result<T>;
