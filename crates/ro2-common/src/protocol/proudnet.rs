//! ProudNet protocol message handlers (opcodes 0x01-0x32)
//!
//! Handles low-level ProudNet protocol messages including:
//! - 0x01: Disconnect notification (graceful close)
//! - 0x2F: Flash policy request (XML response, no framing)
//! - 0x04: Encryption handshake (send RSA public key)
//! - 0x05: Encryption response (receive encrypted AES key)
//! - 0x06: Encryption ready acknowledgment
//! - 0x07: Version check
//! - 0x0A: Connection success (session ID)
//! - 0x1B/0x1D: Heartbeat request/response
//! - 0x1C: Keep-alive ping (no response needed)
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
#[cfg(feature = "server")]
use rsa::pkcs1::EncodeRsaPublicKey;
#[cfg(feature = "server")]
use rsa::traits::PublicKeyParts;
#[cfg(feature = "server")]
use std::net::SocketAddr;
use tracing::{debug, warn};

#[cfg(feature = "server")]
/// Flash cross-domain policy XML
///
/// Sent in response to 0x2F policy request.
/// **Important**: This response has NO ProudNet framing (no 0x5713 magic).
/// The client expects raw XML data with null terminator (110 bytes total).
pub const FLASH_POLICY_XML: &[u8] = b"<?xml version=\"1.0\"?><cross-domain-policy><allow-access-from domain=\"*\" to-ports=\"*\" /></cross-domain-policy>\0";

#[cfg(feature = "server")]
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

#[cfg(feature = "server")]
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

#[cfg(feature = "server")]
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

#[cfg(feature = "server")]
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

    /// Create a new ProudNet handler with a shared RSA keypair
    ///
    /// This allows multiple connections to share the same RSA keypair,
    /// which is necessary for clients that cache server RSA keys.
    pub fn with_shared_crypto(
        remote_addr: SocketAddr,
        settings: ProudNetSettings,
        crypto: std::sync::Arc<ProudNetCrypto>,
    ) -> Self {
        Self {
            crypto: (*crypto).clone(),
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
            0x01 => self.handle_disconnect_notify(payload),
            0x2F => self.handle_policy_request(),
            0x04 => Ok(None), // Client should never send 0x04
            0x05 => self.handle_encryption_response(payload),
            0x07 => self.handle_version_check(payload),
            0x1B => self.handle_heartbeat_request(payload),
            0x1C => self.handle_keep_alive(),
            _ => {
                // Unknown opcode, log and ignore
                debug!(
                    opcode = format!("0x{:02x}", opcode),
                    payload_len = payload.len(),
                    "Unknown ProudNet opcode"
                );
                Ok(None)
            }
        }
    }

    /// Handle 0x01 - Disconnect notification
    ///
    /// Client sends this before closing connection gracefully.
    /// No response is required.
    fn handle_disconnect_notify(&self, payload: &[u8]) -> Result<Option<Vec<u8>>> {
        debug!(
            payload_len = payload.len(),
            "Client disconnect notification (0x01)"
        );
        Ok(None)
    }

    /// Handle 0x2F - Flash policy request
    ///
    /// **Important**: Returns raw XML without ProudNet framing!
    fn handle_policy_request(&self) -> Result<Option<Vec<u8>>> {
        Ok(Some(FLASH_POLICY_XML.to_vec()))
    }

    /// Build 0x04 - Encryption handshake packet (send RSA public key)
    ///
    /// Packet structure (188 bytes total):
    /// ```text
    /// ProudNet Framing (5 bytes):
    ///   13 57           Magic (0x5713 LE)
    ///   02              Size encoding: 2-byte varint
    ///   B7 00           Payload length (183 bytes LE)
    ///
    /// Payload (183 bytes):
    ///   04              Opcode
    ///   
    ///   ProudNet Settings (40 bytes = 10 x u32 LE):
    ///     00 00 00 00   flags
    ///     00 00 00 01   version (0x01000000)
    ///     00 01 00 C0   unknown1 (0x27C00001 mixed endian?)
    ///     27 09 00 01   unknown2 (0x00010927 or typo?)
    ///     00 3C 00 00   timeout_secs (60 = 0x3C)
    ///     00 80 00 00   aes_key_bits (128 = 0x80)
    ///     00 00 02 00   fast_encrypt_key_bits (512 = 0x200)
    ///     00 00 00 01   unknown_flag1 (1)
    ///     00 00 00 01   unknown_flag2 (1)
    ///     00 00 00 02   unknown3 (2)
    ///   
    ///   RSA Public Key:
    ///     8C 00           DER length (140 bytes LE = 0x008C)
    ///     30 81 89 ...    DER-encoded RSA-1024 public key (140 bytes)
    ///                     PKCS#1 ASN.1 structure with modulus and exponent
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

        debug!(
            der_len = der_len,
            key_size_bits = public_key.size() * 8,
            "Built 0x04 encryption handshake packet"
        );

        // Manual framing to match capture format
        // Capture uses 2-byte varint even though payload fits in 1 byte
        let mut packet = Vec::new();
        packet.extend_from_slice(&[0x13, 0x57]); // Magic
        packet.push(0x02); // Size byte: 2-byte varint
        packet.extend_from_slice(&(payload.len() as u16).to_le_bytes()); // Payload size as u16 LE
        packet.extend_from_slice(&payload);

        Ok(packet)
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
            return Err(anyhow!("0x05 payload too short: {} bytes", payload.len()));
        }

        // Parse structure
        let opcode = payload[0]; // Should be 0x05
        let _sub_opcode = payload[1]; // Should be 0x02
        let key_len = u16::from_le_bytes([payload[2], payload[3]]) as usize;

        debug!(
            opcode = format!("0x{:02x}", opcode),
            sub_opcode = format!("0x{:02x}", _sub_opcode),
            key_len = key_len,
            payload_len = payload.len(),
            "Processing 0x05 encryption response"
        );

        if opcode != 0x05 {
            return Err(anyhow!("Expected opcode 0x05, got 0x{:02x}", opcode));
        }

        if payload.len() < 4 + key_len {
            return Err(anyhow!(
                "0x05 payload truncated: have {} bytes, need {}",
                payload.len(),
                4 + key_len
            ));
        }

        // Extract encrypted AES key
        let encrypted_key = &payload[4..4 + key_len];

        // Note: Extra bytes after encrypted key are present in captures but purpose unknown
        // They may be encrypted IV, signature, or protocol metadata
        if payload.len() > 4 + key_len {
            debug!(
                extra_bytes = payload.len() - 4 - key_len,
                "Additional data present after encrypted key"
            );
        }

        // Decrypt the AES session key using our RSA private key
        match self.crypto.decrypt_session_key_rsa(encrypted_key) {
            Ok(session_key) => {
                debug!(
                    session_key_len = session_key.len(),
                    "Successfully decrypted AES session key"
                );

                // LOG SESSION KEY FOR WIRESHARK DECRYPTION
                // Format: AES_SESSION_KEY: <hex>
                // This allows us to decrypt captured traffic later
                if session_key.len() >= 16 {
                    eprintln!(
                        "🔑 AES_SESSION_KEY [{}]: {}",
                        self.remote_addr,
                        hex::encode(&session_key[0..16])
                    );
                }

                // Mark encryption as ready
                self.encryption_ready = true;

                // Send 0x06 (Ready) response
                let response = PacketFrame::new(vec![0x06]);

                Ok(Some(response.to_bytes()))
            }
            Err(e) => {
                warn!(error = %e, "RSA decryption failed");
                Err(anyhow!("Failed to decrypt with RSA: {}", e))
            }
        }
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

        debug!(
            version = version,
            guid = ?&payload[3..19],
            "Client version check"
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
    ///
    /// Client sends this periodically (~5 seconds) with timestamp data.
    /// Server must respond with 0x1D (heartbeat ACK).
    fn handle_heartbeat_request(&self, payload: &[u8]) -> Result<Option<Vec<u8>>> {
        // Send 0x1D (Heartbeat ACK) with extended format (17 bytes total)
        // The client sends a heartbeat with sequence number and expects it echoed back

        // Extract sequence number from client's payload (bytes 0-1 after opcode was already stripped)
        let sequence = if payload.len() >= 2 {
            [payload[0], payload[1]]
        } else {
            [0x00, 0x00]
        };

        // Build extended heartbeat response (17 bytes)
        let mut response_payload = vec![
            0x1D, // Opcode
            sequence[0],
            sequence[1], // Echo client's sequence number
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00, // Reserved (8 bytes)
            0x00,
            0x00,
            0x00,
            0x00, // Unknown field (4 bytes)
            0x00,
            0x00, // Padding (2 bytes)
        ];

        let response = PacketFrame::new(response_payload);
        Ok(Some(response.to_bytes()))
    }

    /// Handle 0x1C - Keep-alive ping
    ///
    /// Client sends this with no payload (just opcode).
    /// No response is required - this is a fire-and-forget keep-alive.
    fn handle_keep_alive(&self) -> Result<Option<Vec<u8>>> {
        debug!("Received keep-alive ping (0x1C)");
        Ok(None)
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

    /// Encrypt a game message payload and wrap in 0x25 packet
    pub fn encrypt_packet(&self, payload: &[u8]) -> Result<Vec<u8>> {
        if !self.encryption_ready {
            return Err(anyhow!("Encryption not ready"));
        }

        // Encrypt the payload
        let encrypted = self.crypto.encrypt_aes_ecb(payload)?;

        // Build 0x25 packet frame
        // Structure: [opcode] [flags:3bytes] [encrypted data]
        // Flags observed from captures: 0x01 0x01 0x20
        let mut packet_data = vec![
            0x25, // Opcode (encrypted message)
            0x01, // Flag byte 1
            0x01, // Flag byte 2
            0x20, // Flag byte 3
        ];

        // Add encrypted data
        packet_data.extend_from_slice(&encrypted);

        // Wrap in ProudNet frame (adds magic + varint size)
        let frame = PacketFrame::new(packet_data);
        Ok(frame.to_bytes())
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[test]
    fn test_policy_request() {
        let handler = ProudNetHandler::new("127.0.0.1:7101".parse().unwrap());
        let response = handler.handle_policy_request().unwrap().unwrap();

        assert_eq!(response, FLASH_POLICY_XML);
        assert_eq!(response.len(), 110); // 109 bytes + null terminator
        assert_eq!(response[response.len() - 1], 0); // Ends with null terminator
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
