//! RO2 Common Library
//!
//! Shared functionality for Ragnarok Online 2 server emulator including:
//! - Protocol definitions (ProudNet RMI)
//! - Packet structures
//! - Cryptography (AES/RSA)
//! - Database models

pub mod crypto;
pub mod database;
pub mod packet;
pub mod protocol;

pub use packet::{NetworkPacket, PacketBuffer, PacketHeader};
pub use protocol::MessageType;

/// Common result type for RO2 operations
pub type Result<T> = anyhow::Result<T>;
