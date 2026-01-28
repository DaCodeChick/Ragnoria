//! ProudNet protocol message handlers (opcodes 0x01-0x32)
//!
//! Handles low-level ProudNet protocol messages including:
//! - 0x2F: Flash policy request (XML response, no framing)
//! - 0x04: Encryption handshake (send RSA public key)
//! - 0x05: Encryption response (receive encrypted AES key)
//! - 0x06: Encryption ready acknowledgment
//! - 0x07: Version check
//! - 0x0A: Connection success (session ID)
//! - 0x1B/0x1D: Heartbeat
//! - 0x25/0x26: Encrypted game messages
//!
//! ## TODO: Settings Structure Research
//!
//! The `ProudNetSettings` structure contains 10 u32 fields that are not fully
//! understood. Current implementation uses known working values.
//!
//! **Known fields:**
//! - `aes_key_bits`: AES key size (confirmed via Ghidra offset +0x638)
//! - `fast_encrypt_key_bits`: Fast encrypt key size (confirmed via Ghidra offset +0x63c)
//!
//! **Research needed:**
//! - What do the unknown fields control?
//! - Are these values static or dynamic?
//! - How does changing them affect client behavior?
//!
//! To investigate:
//! 1. Search for `DeserializeConnectionSettings` function in Ghidra
//! 2. Analyze how each field is used after deserialization
//! 3. Test with modified values to observe client reactions
//! 4. Cross-reference with ProudNet SDK documentation if available

use crate::crypto::ProudNetCrypto;
use crate::packet::framing::PacketFrame;
use anyhow::{anyhow, Result};
use rsa::pkcs1::EncodeRsaPublicKey;
use std::net::SocketAddr;

/// Flash cross-domain policy XML
///
/// Sent in response to 0x2F policy request.
/// **Important**: This response has NO ProudNet framing (no 0x5713 magic).
/// The client expects raw XML data.
pub const FLASH_POLICY_XML: &str = r#"<?xml version="1.0"?>
<cross-domain-policy>
<allow-access-from domain="*" to-ports="*" />
</cross-domain-policy>"#;

/// ProudNet connection settings for 0x04 packet
///
/// These settings are sent during the encryption handshake.
/// Some fields are not fully understood - see field comments for details.
/// These match the structure deserialized by `DeserializeConnectionSettings` in client.
#[derive(Debug, Clone)]
pub struct ProudNetSettings {
    /// Flags (unknown purpose) - observed: 0x00000000
    pub flags: u32,

    /// Protocol version - observed: 0x01000000 (v1)
    pub version: u32,

    /// Unknown setting 1 - observed: 0x27c00001
    pub unknown1: u32,

    /// Unknown setting 2 - observed: 0x00010009
    pub unknown2: u32,

    /// Possibly timeout in seconds - observed: 60 (0x3c)
    pub timeout_secs: u32,

    /// AES key size in BITS - observed: 128 (AES-128)
    /// Client uses this at offset +0x638, divides by 8 for bytes
    pub aes_key_bits: u32,

    /// Fast encrypt key size in BITS - observed: 512
    /// Client uses this at offset +0x63c, divides by 8 for bytes  
    pub fast_encrypt_key_bits: u32,

    /// Unknown flag 1 - observed: 1 (enabled?)
    pub unknown_flag1: u32,

    /// Unknown flag 2 - observed: 1 (enabled?)
    pub unknown_flag2: u32,

    /// Unknown setting 3 - observed: 0x02000000 or 2 (LE ambiguous)
    pub unknown3: u32,
}

impl Default for ProudNetSettings {
    /// Default ProudNet settings
    ///
    /// **WARNING**: These are known working values from protocol analysis.
    /// The actual meaning of most fields is unknown. Use with caution!
    fn default() -> Self {
        Self {
            flags: 0x00000000,
            version: 0x01000000,
            unknown1: 0x27c00001,
            unknown2: 0x00010009,
            timeout_secs: 60,           // Best guess based on value
            aes_key_bits: 128,          // Confirmed via Ghidra analysis
            fast_encrypt_key_bits: 512, // Confirmed via Ghidra analysis
            unknown_flag1: 1,
            unknown_flag2: 1,
            unknown3: 0x02000000, // Could be 2 or 0x02000000 depending on endianness interpretation
        }
    }
}

/// ProudNet protocol handler
///
/// Manages encryption handshake and low-level protocol messages.
/// Each connection has its own instance to track encryption state.
pub struct ProudNetHandler {
    /// Crypto handler for this connection
    crypto: ProudNetCrypto,

    /// Connection address
    remote_addr: SocketAddr,

    /// Session ID (assigned after encryption handshake)
    session_id: Option<u32>,

    /// Encryption established flag
    encryption_ready: bool,

    /// Version from client
    client_version: Option<u32>,

    /// ProudNet settings for this connection
    settings: ProudNetSettings,
}

impl ProudNetHandler {
    /// Create a new ProudNet handler for a connection
    pub fn new(remote_addr: SocketAddr) -> Self {
        Self::with_settings(remote_addr, ProudNetSettings::default())
    }

    /// Create a new ProudNet handler with custom settings
    pub fn with_settings(remote_addr: SocketAddr, settings: ProudNetSettings) -> Self {
        let mut crypto = ProudNetCrypto::new();

        // Generate RSA keypair (1024-bit as used by RO2)
        crypto
            .generate_rsa_keypair(1024)
            .expect("Failed to generate RSA keypair");

        Self {
            crypto,
            remote_addr,
            session_id: None,
            encryption_ready: false,
            client_version: None,
            settings,
        }
    }

    /// Handle ProudNet protocol message
    ///
    /// Returns response bytes (may or may not have ProudNet framing)
    pub fn handle(&mut self, opcode: u8, payload: &[u8]) -> Result<Option<Vec<u8>>> {
        match opcode {
            0x2F => self.handle_policy_request(),
            0x04 => Ok(None), // Client should never send 0x04
            0x05 => self.handle_encryption_response(payload),
            0x07 => self.handle_version_check(payload),
            0x1B => self.handle_heartbeat_request(payload),
            _ => {
                // Unknown opcode, ignore
                Ok(None)
            }
        }
    }

    /// Handle 0x2F - Flash policy request
    ///
    /// **Important**: Returns raw XML without ProudNet framing!
    fn handle_policy_request(&self) -> Result<Option<Vec<u8>>> {
        Ok(Some(FLASH_POLICY_XML.as_bytes().to_vec()))
    }

    /// Build 0x04 - Encryption handshake (send RSA public key)
    ///
    /// This should be sent immediately after the policy response.
    ///
    /// Packet structure (183 bytes total):
    /// ```text
    /// 04 [40 bytes of settings] [2-byte DER length] [DER-encoded RSA public key]
    /// │
    /// └─ Opcode
    ///
    /// Settings (10 x u32 = 40 bytes):
    /// - See ProudNetSettings structure for field descriptions
    /// - Most fields are not fully understood yet
    /// - AES key size and fast encrypt key size are confirmed
    /// ```
    pub fn build_encryption_handshake(&self) -> Result<Vec<u8>> {
        let mut payload = Vec::new();

        // Opcode
        payload.push(0x04);

        // Settings (10 x u32 = 40 bytes)
        // Use the settings from this handler instance
        let s = &self.settings;
        payload.extend_from_slice(&s.flags.to_le_bytes());
        payload.extend_from_slice(&s.version.to_le_bytes());
        payload.extend_from_slice(&s.unknown1.to_le_bytes());
        payload.extend_from_slice(&s.unknown2.to_le_bytes());
        payload.extend_from_slice(&s.timeout_secs.to_le_bytes());
        payload.extend_from_slice(&s.aes_key_bits.to_le_bytes());
        payload.extend_from_slice(&s.fast_encrypt_key_bits.to_le_bytes());
        payload.extend_from_slice(&s.unknown_flag1.to_le_bytes());
        payload.extend_from_slice(&s.unknown_flag2.to_le_bytes());
        payload.extend_from_slice(&s.unknown3.to_le_bytes());

        // Get RSA public key in DER format
        let public_key = self
            .crypto
            .rsa_public_key()
            .ok_or_else(|| anyhow!("No RSA public key"))?;

        let der_bytes = public_key
            .to_pkcs1_der()
            .map_err(|e| anyhow!("Failed to encode RSA key: {}", e))?;

        // DER length as u16 LE
        let der_len = der_bytes.as_bytes().len() as u16;
        payload.extend_from_slice(&der_len.to_le_bytes());

        // DER-encoded public key
        payload.extend_from_slice(der_bytes.as_bytes());

        // Wrap in PacketFrame
        let frame = PacketFrame::new(payload);

        Ok(frame.to_bytes())
    }

    /// Handle 0x05 - Encryption response (client sends encrypted AES key)
    ///
    /// Structure:
    /// ```text
    /// 05 02 8000 [128 bytes of RSA-encrypted AES session key] [additional encrypted data]
    /// │  │  │
    /// │  │  └─ Key length (u16 LE = 0x0080 = 128 bytes)
    /// │  └─ Sub-opcode
    /// └─ Opcode
    /// ```
    fn handle_encryption_response(&mut self, payload: &[u8]) -> Result<Option<Vec<u8>>> {
        if payload.len() < 5 {
            return Err(anyhow!("0x05 payload too short"));
        }

        // Parse structure
        let opcode = payload[0]; // Should be 0x05
        let sub_opcode = payload[1]; // Should be 0x02
        let key_len = u16::from_le_bytes([payload[2], payload[3]]) as usize;

        if opcode != 0x05 {
            return Err(anyhow!("Expected opcode 0x05, got 0x{:02x}", opcode));
        }

        if payload.len() < 4 + key_len {
            return Err(anyhow!("0x05 payload truncated"));
        }

        // Extract encrypted AES key
        let encrypted_key = &payload[4..4 + key_len];

        // Decrypt the AES session key using our RSA private key
        let session_key = self.crypto.decrypt_session_key_rsa(encrypted_key)?;

        println!(
            "[ProudNet] Decrypted AES session key: {} bytes",
            session_key.len()
        );

        // Mark encryption as ready
        self.encryption_ready = true;

        // Send 0x06 (Ready) response
        let response = PacketFrame::new(vec![0x06]);

        Ok(Some(response.to_bytes()))
    }

    /// Handle 0x07 - Version check
    ///
    /// Structure:
    /// ```text
    /// 07 0100 [16 bytes GUID] 010300
    /// │  │    │               │
    /// │  Ver  Client GUID     Flags
    /// └─ Opcode
    /// ```
    fn handle_version_check(&mut self, payload: &[u8]) -> Result<Option<Vec<u8>>> {
        if payload.len() < 23 {
            return Err(anyhow!("0x07 payload too short"));
        }

        let version = u16::from_le_bytes([payload[1], payload[2]]);
        self.client_version = Some(version as u32);

        println!(
            "[ProudNet] Client version: {}, GUID: {:02x?}",
            version,
            &payload[3..19]
        );

        // Generate session ID
        self.session_id = Some(rand::random::<u32>());

        // Send 0x0A (Connection success with session ID)
        self.build_connection_success()
    }

    /// Build 0x0A - Connection success response
    ///
    /// Structure:
    /// ```text
    /// 0a [session_id: u32] [server_guid: 16 bytes] 0100 01 01 [ip_len: u8] [ip_string] [crc: u16]
    /// ```
    fn build_connection_success(&self) -> Result<Option<Vec<u8>>> {
        let mut payload = Vec::new();

        // Opcode
        payload.push(0x0A);

        // Session ID
        let session_id = self.session_id.unwrap_or(0);
        payload.extend_from_slice(&session_id.to_le_bytes());

        // Server GUID (16 random bytes)
        let server_guid: [u8; 16] = rand::random();
        payload.extend_from_slice(&server_guid);

        // Flags
        payload.extend_from_slice(&[0x01, 0x00]); // u16 LE
        payload.push(0x01);
        payload.push(0x01);

        // Server IP address (use connection address)
        let ip_str = self.remote_addr.ip().to_string();
        payload.push(ip_str.len() as u8);
        payload.extend_from_slice(ip_str.as_bytes());

        // CRC/checksum field (purpose unclear)
        payload.extend_from_slice(&[0xac, 0xf6]);

        let frame = PacketFrame::new(payload);

        Ok(Some(frame.to_bytes()))
    }

    /// Handle 0x1B - Heartbeat request
    fn handle_heartbeat_request(&self, _payload: &[u8]) -> Result<Option<Vec<u8>>> {
        // Send 0x1D (Heartbeat ACK)
        let response = PacketFrame::new(vec![0x1D]);

        Ok(Some(response.to_bytes()))
    }

    /// Check if encryption is ready
    pub fn is_encryption_ready(&self) -> bool {
        self.encryption_ready
    }

    /// Get session ID
    pub fn session_id(&self) -> Option<u32> {
        self.session_id
    }

    /// Decrypt an encrypted packet (0x25/0x26)
    pub fn decrypt_packet(&self, payload: &[u8]) -> Result<Vec<u8>> {
        if !self.encryption_ready {
            return Err(anyhow!("Encryption not ready"));
        }

        self.crypto.decrypt_packet_0x25(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_request() {
        let handler = ProudNetHandler::new("127.0.0.1:7101".parse().unwrap());
        let response = handler.handle_policy_request().unwrap().unwrap();

        assert_eq!(response, FLASH_POLICY_XML.as_bytes());
    }

    #[test]
    fn test_encryption_handshake_structure() {
        let handler = ProudNetHandler::new("127.0.0.1:7101".parse().unwrap());
        let packet = handler.build_encryption_handshake().unwrap();

        // Check ProudNet magic
        assert_eq!(&packet[0..2], &[0x13, 0x57]);

        // Parse frame to check payload
        let (frame, _) = PacketFrame::from_bytes(&packet).unwrap();
        let payload = frame.payload;

        // Check opcode
        assert_eq!(payload[0], 0x04);

        // Check settings (40 bytes after opcode)
        assert!(payload.len() >= 41);

        // Check DER marker at offset 43 (after 1 byte opcode + 40 bytes settings + 2 bytes length)
        // DER should start with 0x30 (SEQUENCE)
        assert_eq!(payload[43], 0x30);
    }
}
