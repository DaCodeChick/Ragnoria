//! ProudNet encryption and decryption
//!
//! ProudNet protocol uses dual-layer encryption:
//! 1. RSA for session key exchange (opcodes 0x04, 0x05)
//! 2. AES for encrypting game messages (opcode 0x25)
//!
//! Encryption flow:
//! 1. Server sends RSA public key (0x04)
//! 2. Client generates AES session key
//! 3. Client encrypts session key with RSA and sends it (0x05)
//! 4. Server decrypts session key with RSA private key
//! 5. All subsequent game messages encrypted with AES in 0x25 packets

use crate::Result;
use aes::Aes128;
use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray};
use rand::{Rng, rngs::OsRng};
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::{Oaep, Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use sha1::Sha1;
use sha2::Sha256;
use tracing::{debug, warn};

/// ProudNet encryption handler
///
/// Manages RSA and AES encryption for the ProudNet protocol layer.
#[derive(Clone)]
pub struct ProudNetCrypto {
    /// RSA public key (received from server in 0x04 packet)
    rsa_public: Option<RsaPublicKey>,

    /// RSA private key (server-side only)
    rsa_private: Option<RsaPrivateKey>,

    /// AES session key (16 bytes for AES-128)
    aes_key: Option<[u8; 16]>,

    /// AES IV (initialization vector, if using CBC mode)
    aes_iv: Option<[u8; 16]>,
}

impl ProudNetCrypto {
    /// Create a new crypto handler
    pub fn new() -> Self {
        Self {
            rsa_public: None,
            rsa_private: None,
            aes_key: None,
            aes_iv: None,
        }
    }

    /// Parse RSA public key from DER-encoded data
    ///
    /// The server sends an ASN.1 DER encoded RSA public key in the 0x04 packet.
    /// The key starts at offset 0x28 in the payload.
    pub fn set_rsa_public_key_from_der(&mut self, der_data: &[u8]) -> Result<()> {
        let public_key = RsaPublicKey::from_pkcs1_der(der_data)
            .map_err(|e| anyhow::anyhow!("Failed to parse RSA public key: {}", e))?;

        self.rsa_public = Some(public_key);
        Ok(())
    }

    #[cfg(feature = "server")]
    /// Set RSA private key (server-side)
    pub fn set_rsa_private_key(&mut self, private_key: RsaPrivateKey) {
        self.rsa_private = Some(private_key);
    }

    #[cfg(feature = "server")]
    /// Generate a new RSA keypair (server-side)
    pub fn generate_rsa_keypair(&mut self, bits: usize) -> Result<()> {
        let mut rng = OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, bits)
            .map_err(|e| anyhow::anyhow!("Failed to generate RSA keypair: {}", e))?;
        let public_key = RsaPublicKey::from(&private_key);

        self.rsa_private = Some(private_key);
        self.rsa_public = Some(public_key);
        Ok(())
    }

    /// Get RSA public key
    pub fn rsa_public_key(&self) -> Option<&RsaPublicKey> {
        self.rsa_public.as_ref()
    }

    /// Generate AES session key
    pub fn generate_aes_session_key(&mut self) -> [u8; 16] {
        let mut rng = OsRng;
        let mut key = [0u8; 16];
        rng.fill(&mut key);
        self.aes_key = Some(key);
        key
    }

    /// Set AES session key
    pub fn set_aes_session_key(&mut self, key: [u8; 16]) {
        self.aes_key = Some(key);
    }

    /// Get AES session key
    pub fn aes_session_key(&self) -> Option<&[u8; 16]> {
        self.aes_key.as_ref()
    }

    /// Set AES IV (for CBC mode)
    pub fn set_aes_iv(&mut self, iv: [u8; 16]) {
        self.aes_iv = Some(iv);
    }

    /// Encrypt session key with RSA (client-side, opcode 0x05)
    ///
    /// The client encrypts the AES session key with the server's RSA public key
    /// and sends it in a 0x05 packet.
    pub fn encrypt_session_key_rsa(&self, session_key: &[u8]) -> Result<Vec<u8>> {
        let public_key = self
            .rsa_public
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No RSA public key set"))?;

        let mut rng = OsRng;
        let encrypted = public_key
            .encrypt(&mut rng, Pkcs1v15Encrypt, session_key)
            .map_err(|e| anyhow::anyhow!("Failed to encrypt with RSA: {}", e))?;

        Ok(encrypted)
    }

    #[cfg(feature = "server")]
    /// Decrypt session key with RSA (server-side, opcode 0x05)
    ///
    /// RO2 client uses OAEP-SHA1 padding (circa 2011), not PKCS#1 v1.5.
    /// This was discovered through Ghidra analysis of RSA_ApplyPadding function.
    pub fn decrypt_session_key_rsa(&mut self, encrypted_key: &[u8]) -> Result<Vec<u8>> {
        use rsa::traits::PublicKeyParts;

        let private_key = self
            .rsa_private
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No RSA private key set"))?;

        debug!(
            key_size_bits = private_key.size() * 8,
            encrypted_len = encrypted_key.len(),
            "Attempting RSA decryption"
        );

        // RO2 uses OAEP-SHA1 (discovered via Ghidra analysis of RSA_ApplyPadding)
        // Try fallback schemes in case other clients behave differently
        let decrypted = private_key
            .decrypt(Oaep::new::<Sha1>(), encrypted_key)
            .or_else(|e1| {
                debug!(error = %e1, "OAEP-SHA1 failed, trying PKCS#1 v1.5");
                private_key.decrypt(Pkcs1v15Encrypt, encrypted_key)
            })
            .or_else(|e2| {
                debug!(error = %e2, "PKCS#1 v1.5 failed, trying OAEP-SHA256");
                private_key.decrypt(Oaep::new::<Sha256>(), encrypted_key)
            })
            .map_err(|e| {
                warn!(
                    encrypted_len = encrypted_key.len(),
                    key_size = private_key.size() * 8,
                    error = %e,
                    "All RSA decryption methods failed"
                );
                anyhow::anyhow!("Failed to decrypt session key with RSA: {}", e)
            })?;

        debug!(decrypted_len = decrypted.len(), "RSA decryption successful");

        // Extract the 16-byte AES key
        if decrypted.len() >= 16 {
            let mut key = [0u8; 16];
            key.copy_from_slice(&decrypted[0..16]);
            self.aes_key = Some(key);
            debug!("AES session key extracted");
        } else {
            warn!(
                decrypted_len = decrypted.len(),
                "Decrypted data too short for AES key"
            );
        }

        Ok(decrypted)
    }

    /// Encrypt data with AES-128 ECB (block cipher, no IV)
    ///
    /// Note: We need to determine the actual AES mode used by inspecting
    /// encrypted packets. ECB is the simplest (each block encrypted independently).
    /// ProudNet might use CBC, CTR, or another mode.
    pub fn encrypt_aes_ecb(&self, data: &[u8]) -> Result<Vec<u8>> {
        let key = self
            .aes_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No AES session key set"))?;

        let cipher = Aes128::new(GenericArray::from_slice(key));

        // Pad to 16-byte blocks (PKCS#7 padding)
        let mut padded = data.to_vec();
        let padding_len = 16 - (data.len() % 16);
        padded.extend(vec![padding_len as u8; padding_len]);

        // Encrypt each block
        let mut encrypted = Vec::with_capacity(padded.len());
        for chunk in padded.chunks(16) {
            let mut block = GenericArray::clone_from_slice(chunk);
            cipher.encrypt_block(&mut block);
            encrypted.extend_from_slice(&block);
        }

        Ok(encrypted)
    }

    /// Decrypt data with AES-128 ECB
    pub fn decrypt_aes_ecb(&self, data: &[u8]) -> Result<Vec<u8>> {
        let key = self
            .aes_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No AES session key set"))?;

        if !data.len().is_multiple_of(16) {
            return Err(anyhow::anyhow!(
                "Invalid AES data length: {} (must be multiple of 16)",
                data.len()
            ));
        }

        let cipher = Aes128::new(GenericArray::from_slice(key));

        // Decrypt each block
        let mut decrypted = Vec::with_capacity(data.len());
        for chunk in data.chunks(16) {
            let mut block = GenericArray::clone_from_slice(chunk);
            cipher.decrypt_block(&mut block);
            decrypted.extend_from_slice(&block);
        }

        // Remove PKCS#7 padding
        if let Some(&padding_len) = decrypted.last()
            && padding_len > 0 && padding_len <= 16 {
                let len = decrypted.len();
                decrypted.truncate(len - padding_len as usize);
            }

        Ok(decrypted)
    }

    /// Decrypt a 0x25 encrypted packet
    ///
    /// Packet structure:
    /// - Byte 0: 0x25 (opcode)
    /// - Byte 1: Sub-opcode (0x01 or 0x02)
    /// - Byte 2-3: Possible length field?
    /// - Byte 4+: Encrypted data
    pub fn decrypt_packet_0x25(&self, payload: &[u8]) -> Result<Vec<u8>> {
        if payload.is_empty() || payload[0] != 0x25 {
            return Err(anyhow::anyhow!("Not a 0x25 packet"));
        }

        if payload.len() < 4 {
            return Err(anyhow::anyhow!("0x25 packet too short"));
        }

        // Extract encrypted data (skip opcode, sub-opcode, and length field)
        let encrypted_data = &payload[4..];

        // Try to decrypt with AES ECB
        self.decrypt_aes_ecb(encrypted_data)
    }

    // ===== Client-side Convenience Methods =====
    // These are aliases for clearer client code when experimenting with client implementations

    #[cfg(feature = "client")]
    /// Set server public key from DER (client-side)
    /// Alias for set_rsa_public_key_from_der for clearer client code
    pub fn set_server_public_key(&mut self, der_data: &[u8]) -> Result<()> {
        self.set_rsa_public_key_from_der(der_data)
    }

    #[cfg(feature = "client")]
    /// Encrypt session key (client-side)
    /// Alias for encrypt_session_key_rsa for clearer client code
    pub fn encrypt_session_key(&self, session_key: &[u8]) -> Result<Vec<u8>> {
        self.encrypt_session_key_rsa(session_key)
    }

    #[cfg(feature = "client")]
    /// Set session key (client-side)
    /// Alias for set_aes_session_key for clearer client code
    pub fn set_session_key(&mut self, key: [u8; 16]) -> Result<()> {
        self.set_aes_session_key(key);
        Ok(())
    }

    #[cfg(feature = "client")]
    /// Encrypt data for sending (client-side)
    /// Alias for encrypt_aes_ecb for clearer client code
    pub fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        self.encrypt_aes_ecb(data)
    }
}

impl Default for ProudNetCrypto {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_aes_encryption_roundtrip() {
        let mut crypto = ProudNetCrypto::new();
        crypto.generate_aes_session_key();

        let plaintext = b"Hello, RO2 Server!";
        let encrypted = crypto.encrypt_aes_ecb(plaintext).unwrap();
        let decrypted = crypto.decrypt_aes_ecb(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    #[cfg(feature = "server")]
    fn test_rsa_session_key_exchange() {
        use rsa::traits::PublicKeyParts;

        // Server generates keypair
        let mut server = ProudNetCrypto::new();
        server.generate_rsa_keypair(1024).unwrap();

        // Print DER structure
        let der = server.rsa_public_key().unwrap().to_pkcs1_der().unwrap();
        let der_bytes = der.as_bytes();
        println!("\n=== Generated RSA-1024 Key ===");
        println!("DER length: {} bytes", der_bytes.len());
        println!("DER structure:");
        println!("  [0]: 0x{:02x} (SEQUENCE tag)", der_bytes[0]);
        println!("  [1]: 0x{:02x} (long-form length marker)", der_bytes[1]);
        println!("  [2]: 0x{:02x} ({} bytes)", der_bytes[2], der_bytes[2]);
        println!("  [3]: 0x{:02x} (INTEGER tag for modulus)", der_bytes[3]);
        println!("  [4]: 0x{:02x} (long-form length marker)", der_bytes[4]);
        println!("  [5]: 0x{:02x} ({} bytes)", der_bytes[5], der_bytes[5]);
        println!("  [6]: 0x{:02x} (leading zero byte)", der_bytes[6]);
        println!("\nDER (first 32 bytes): {}", hex::encode(&der_bytes[..32]));
        println!(
            "Modulus size: {} bits",
            server.rsa_public_key().unwrap().size() * 8
        );
        println!("Exponent: 0x{:x}\n", server.rsa_public_key().unwrap().e());

        // Client receives public key and generates session key
        let mut client = ProudNetCrypto::new();
        client.set_rsa_public_key_from_der(der.as_bytes()).unwrap();

        let session_key = client.generate_aes_session_key();

        // Client encrypts session key with RSA
        let encrypted_key = client.encrypt_session_key_rsa(&session_key).unwrap();

        // Server decrypts session key with RSA
        let decrypted_key = server.decrypt_session_key_rsa(&encrypted_key).unwrap();

        // Session keys should match (at least first 16 bytes)
        assert_eq!(&decrypted_key[0..16], &session_key[..]);
        assert_eq!(server.aes_session_key(), Some(&session_key));
    }

    #[test]
    fn test_aes_block_sizes() {
        let mut crypto = ProudNetCrypto::new();
        crypto.generate_aes_session_key();

        // Test various data sizes
        for size in &[1, 15, 16, 17, 31, 32, 100] {
            let plaintext: Vec<u8> = (0..*size).map(|_| rand::random()).collect();

            let encrypted = crypto.encrypt_aes_ecb(&plaintext).unwrap();
            let decrypted = crypto.decrypt_aes_ecb(&encrypted).unwrap();

            assert_eq!(decrypted, plaintext, "Failed for size {}", size);

            // Encrypted data should be padded to 16-byte blocks
            assert_eq!(encrypted.len() % 16, 0);
        }
    }

    #[test]
    #[cfg(feature = "server")]
    fn test_rsa_decrypt_raw_data() {
        use rsa::traits::PublicKeyParts;

        // Create a keypair
        let mut server = ProudNetCrypto::new();
        server.generate_rsa_keypair(1024).unwrap();

        // Get the keys
        let public_key = server.rsa_public_key().unwrap();
        let private_key = server.rsa_private.as_ref().unwrap();

        println!("\n=== RSA Raw Encrypt/Decrypt Test ===");
        println!(
            "Public key modulus (hex): {}",
            hex::encode(public_key.n().to_bytes_be())
        );
        println!("Public key exponent: {}", public_key.e());

        // Test data (16 bytes like AES key)
        let test_data = b"0123456789ABCDEF";
        println!("\nOriginal data: {}", hex::encode(test_data));

        // Encrypt
        let mut rng = rand::rngs::OsRng;
        let encrypted = public_key
            .encrypt(&mut rng, Pkcs1v15Encrypt, test_data)
            .unwrap();
        println!(
            "Encrypted ({} bytes): {}",
            encrypted.len(),
            hex::encode(&encrypted)
        );

        // Decrypt
        let decrypted = private_key.decrypt(Pkcs1v15Encrypt, &encrypted).unwrap();
        println!("Decrypted: {}", hex::encode(&decrypted));

        assert_eq!(test_data, &decrypted[..]);
        println!("✓ Test passed!");
    }
}

#[test]
#[cfg(feature = "server")]
fn test_rsa_keypair_consistency() {
    use rsa::traits::PublicKeyParts;

    println!("\n=== RSA Keypair Consistency Test ===");

    // Generate a keypair
    let mut crypto = ProudNetCrypto::new();
    crypto.generate_rsa_keypair(1024).unwrap();

    // Get the keys
    let public_key = crypto.rsa_public_key().unwrap();
    let private_key = crypto.rsa_private.as_ref().unwrap();

    // Verify modulus matches
    let pub_n = public_key.n();
    let priv_n = private_key.n();

    println!("Public key modulus:  {}", hex::encode(pub_n.to_bytes_be()));
    println!("Private key modulus: {}", hex::encode(priv_n.to_bytes_be()));

    assert_eq!(
        pub_n, priv_n,
        "Modulus mismatch between public and private key!"
    );
    println!("✓ Moduli match!");

    // Verify exponents
    println!("Public exponent: {}", public_key.e());
    println!("Private exponent (d): <hidden>");

    // Now test encrypt/decrypt cycle
    let test_data = b"Test session key";
    let mut rng = rand::rngs::OsRng;

    let encrypted = public_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, test_data)
        .unwrap();
    println!(
        "\nEncrypted {} bytes to {} bytes",
        test_data.len(),
        encrypted.len()
    );

    let decrypted = private_key.decrypt(Pkcs1v15Encrypt, &encrypted).unwrap();
    println!("Decrypted back to {} bytes", decrypted.len());

    assert_eq!(test_data, &decrypted[..], "Decryption mismatch!");
    println!("✓ Encrypt/decrypt cycle works!");
}
