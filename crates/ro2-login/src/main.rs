//! RO2 Login Server
//!
//! Handles client authentication on port 7101

mod handlers;

use anyhow::Result;
use ro2_common::crypto::ProudNetCrypto;
use ro2_common::packet::framing::PacketFrame;
use ro2_common::protocol::{ProudNetHandler, ProudNetSettings};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info, warn};

const LOGIN_PORT: u16 = 7101;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("==============================================");
    info!("   RO2 Login Server v{}", env!("CARGO_PKG_VERSION"));
    info!("==============================================");
    info!("");
    info!("Protocol: ProudNet with RSA-1024 + AES-128");
    info!("Port: {}", LOGIN_PORT);
    info!("");

    // Generate server RSA keypair (shared across all connections)
    info!("Generating server RSA-1024 keypair...");
    let mut server_crypto = ProudNetCrypto::new();
    server_crypto.generate_rsa_keypair(1024)?;
    let server_crypto = Arc::new(server_crypto);
    info!("✓ RSA keypair generated");
    info!("");

    // TODO: Initialize database connection
    // let db = setup_database().await?;

    // Bind to login port
    let addr = SocketAddr::from(([0, 0, 0, 0], LOGIN_PORT));
    let listener = TcpListener::bind(addr).await?;

    info!("Login server listening on {}", addr);
    info!("Waiting for connections...");
    info!("==============================================");
    info!("");

    // Accept connections
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                info!("New connection from {}", addr);

                // Clone Arc for this connection
                let crypto = Arc::clone(&server_crypto);

                // Spawn a task to handle this client
                tokio::spawn(async move {
                    if let Err(e) = handle_client(socket, addr, crypto).await {
                        error!("Error handling client {}: {}", addr, e);
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

/// Connection state for a single client
struct ClientConnection {
    stream: TcpStream,
    addr: SocketAddr,
    handler: ProudNetHandler,
    buffer: Vec<u8>,
}

impl ClientConnection {
    fn new(stream: TcpStream, addr: SocketAddr, crypto: Arc<ProudNetCrypto>) -> Self {
        let settings = ProudNetSettings::default();
        info!(
            "[{}] ProudNet settings: AES-{}, Fast-{}, Version: 0x{:08x}",
            addr, settings.aes_key_bits, settings.fast_encrypt_key_bits, settings.version
        );

        Self {
            stream,
            addr,
            handler: ProudNetHandler::with_shared_crypto(addr, settings, crypto),
            buffer: Vec::new(),
        }
    }

    /// Handle the client connection
    async fn handle(&mut self) -> Result<()> {
        let mut read_buf = vec![0u8; 4096];

        loop {
            // Read data from client
            let n = match self.stream.read(&mut read_buf).await {
                Ok(0) => {
                    info!("[{}] Client disconnected", self.addr);
                    return Ok(());
                }
                Ok(n) => n,
                Err(e) => {
                    error!("[{}] Read error: {}", self.addr, e);
                    return Err(e.into());
                }
            };

            // Add to buffer
            self.buffer.extend_from_slice(&read_buf[..n]);
            info!(
                "[{}] Received {} bytes (buffer: {})",
                self.addr,
                n,
                self.buffer.len()
            );

            // Try to parse packets
            self.process_buffer().await?;
        }
    }

    /// Process buffered data and parse packets
    async fn process_buffer(&mut self) -> Result<()> {
        loop {
            // Try to parse ProudNet packet
            if self.buffer.len() < 4 {
                // Need at least magic + size byte
                break;
            }

            // Check for ProudNet magic
            if &self.buffer[0..2] != &[0x13, 0x57] {
                error!(
                    "[{}] Invalid packet magic: {:02x} {:02x}",
                    self.addr, self.buffer[0], self.buffer[1]
                );
                error!(
                    "[{}] Buffer: {}",
                    self.addr,
                    hex::encode(&self.buffer[..self.buffer.len().min(64)])
                );
                self.buffer.clear(); // Discard invalid data
                break;
            }

            // Try to parse packet
            match PacketFrame::from_bytes(&self.buffer) {
                Ok((packet, size)) => {
                    // Remove parsed bytes from buffer
                    self.buffer.drain(..size);

                    // Process packet
                    self.handle_packet(packet).await?;
                }
                Err(e) => {
                    // Check if it's just incomplete
                    if e.to_string().contains("Incomplete packet") {
                        // Need more data
                        break;
                    } else {
                        error!("[{}] Packet parse error: {}", self.addr, e);
                        self.buffer.clear();
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle a parsed ProudNet packet
    async fn handle_packet(&mut self, packet: PacketFrame) -> Result<()> {
        let opcode = packet.opcode().unwrap_or(0);

        info!(
            "[{}] Packet: opcode=0x{:02x}, size={}",
            self.addr,
            opcode,
            packet.payload.len()
        );

        // Handle based on opcode
        match opcode {
            0x01 => {
                info!("[{}] 0x01: Disconnect notification", self.addr);
                self.handler.handle(0x01, &packet.payload)?;
                // Client is closing - we can gracefully terminate
            }

            0x2F => {
                info!("[{}] 0x2F: Policy request", self.addr);
                if let Some(response) = self.handler.handle(0x2F, &packet.payload)? {
                    info!(
                        "[{}] Sending XML policy ({} bytes)",
                        self.addr,
                        response.len()
                    );
                    self.stream.write_all(&response).await?;
                    self.stream.flush().await?;

                    // Send 0x04 encryption handshake
                    info!("[{}] 0x04: Sending encryption handshake", self.addr);
                    let handshake = self.handler.build_encryption_handshake()?;
                    info!(
                        "[{}] 0x04: Packet size: {} bytes",
                        self.addr,
                        handshake.len()
                    );
                    info!(
                        "[{}] 0x04: First 32 bytes: {}",
                        self.addr,
                        hex::encode(&handshake[..32.min(handshake.len())])
                    );
                    self.stream.write_all(&handshake).await?;
                    self.stream.flush().await?;
                }
            }

            0x05 => {
                info!("[{}] 0x05: Encryption response", self.addr);
                match self.handler.handle(0x05, &packet.payload) {
                    Ok(Some(response)) => {
                        info!("[{}] 0x06: Sending encryption ready", self.addr);
                        self.stream.write_all(&response).await?;
                        self.stream.flush().await?;
                    }
                    Ok(None) => {
                        warn!("[{}] 0x05: No response generated", self.addr);
                    }
                    Err(e) => {
                        error!("[{}] 0x05: Failed to decrypt session key: {}", self.addr, e);
                        // Don't disconnect - just log the error for debugging
                    }
                }
            }

            0x07 => {
                info!("[{}] 0x07: Version check", self.addr);
                if let Some(response) = self.handler.handle(0x07, &packet.payload)? {
                    let session_id = self.handler.session_id().unwrap_or(0);
                    info!(
                        "[{}] 0x0A: Sending connection success (session: {})",
                        self.addr, session_id
                    );
                    self.stream.write_all(&response).await?;
                    self.stream.flush().await?;
                }
            }

            0x1B => {
                info!("[{}] 0x1B: Heartbeat", self.addr);
                if let Some(response) = self.handler.handle(0x1B, &packet.payload)? {
                    info!("[{}] 0x1D: Sending heartbeat ack", self.addr);
                    self.stream.write_all(&response).await?;
                    self.stream.flush().await?;
                }
            }

            0x1C => {
                info!("[{}] 0x1C: Keep-alive ping", self.addr);
                if let Some(response) = self.handler.handle(0x1C, &packet.payload)? {
                    self.stream.write_all(&response).await?;
                    self.stream.flush().await?;
                }
            }

            0x25 | 0x26 => {
                info!("[{}] 0x{:02x}: Encrypted packet", self.addr, opcode);

                if !self.handler.is_encryption_ready() {
                    warn!("[{}] Encryption not ready yet, cannot decrypt", self.addr);
                    return Ok(());
                }

                // Decrypt the packet
                match self.handler.decrypt_packet(&packet.payload) {
                    Ok(decrypted) => {
                        info!(
                            "[{}] Decrypted {} bytes: {}",
                            self.addr,
                            decrypted.len(),
                            hex::encode(&decrypted[..decrypted.len().min(32)])
                        );

                        // Check if it's a game message (opcode >= 0x1000)
                        if decrypted.len() >= 2 {
                            let game_opcode = u16::from_le_bytes([decrypted[0], decrypted[1]]);
                            info!(
                                "[{}] GAME MESSAGE: 0x{:04x} ({} bytes total)",
                                self.addr,
                                game_opcode,
                                decrypted.len()
                            );

                            // TODO: Route to game message handlers
                            match game_opcode {
                                0x0000 => {
                                    info!(
                                        "[{}] Game message 0x0000: Initial handshake",
                                        self.addr
                                    );
                                    info!(
                                        "[{}] Full payload: {}",
                                        self.addr,
                                        hex::encode(&decrypted)
                                    );
                                    
                                    // Client packet structure (26 bytes):
                                    // 0x00-0x01: Opcode 0x0000
                                    // 0x02-0x03: 0x01E1 (version/build?)
                                    // 0x04-0x05: 0x2E10 (4142 decimal - another version?)
                                    // 0x06-0x07: 0x0021
                                    // 0x08-0x0B: 0xCBA416F1 (timestamp/GUID?)
                                    // 0x0C-0x0D: 0x0001
                                    // 0x0E-0x11: 0x00000001 (capability flags?)
                                    // 0x12-0x15: 0x07022500 
                                    // 0x16-0x19: 0x803F0000 (float 1.0 in LE: 00 00 80 3f)
                                    
                                    // Generate a server GUID (use timestamp)
                                    use std::time::{SystemTime, UNIX_EPOCH};
                                    let server_guid = SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap()
                                        .as_secs() as u32;
                                    
                                    info!("[{}] Sending 0x0000 server response", self.addr);
                                    
                                    // Extract client's values to mirror them EXACTLY
                                    let client_version = if decrypted.len() >= 4 {
                                        [decrypted[2], decrypted[3]]
                                    } else {
                                        [0x01, 0xE1]
                                    };
                                    let client_build = if decrypted.len() >= 6 {
                                        [decrypted[4], decrypted[5]]
                                    } else {
                                        [0x2E, 0x10]
                                    };
                                    let client_field1 = if decrypted.len() >= 8 {
                                        [decrypted[6], decrypted[7]]
                                    } else {
                                        [0x00, 0x21]
                                    };
                                    let client_field2 = if decrypted.len() >= 14 {
                                        [decrypted[12], decrypted[13]]
                                    } else {
                                        [0x00, 0x01]
                                    };
                                    let client_field3 = if decrypted.len() >= 20 {
                                        [decrypted[18], decrypted[19], decrypted[20], decrypted[21]]
                                    } else {
                                        [0x07, 0x02, 0x25, 0x00]
                                    };
                                    // CRITICAL TEST: Mirror client's exact value
                                    let client_field4 = if decrypted.len() >= 26 {
                                        [decrypted[22], decrypted[23], decrypted[24], decrypted[25]]
                                    } else {
                                        [0x80, 0x3F, 0x00, 0x00]
                                    };
                                    
                                    info!("[{}] TESTING: Mirroring client's 0x803F0000 exactly", self.addr);
                                    
                                    // Extract the "status" field from client (bytes 14-17)
                                    // CRITICAL FIX: Client sends 0x00000001 here, we MUST mirror it!
                                    let client_status = if decrypted.len() >= 18 {
                                        [decrypted[14], decrypted[15], decrypted[16], decrypted[17]]
                                    } else {
                                        [0x00, 0x00, 0x00, 0x01]
                                    };
                                    
                                    // Server should send its OWN GUID, not mirror client's
                                    let guid_bytes = server_guid.to_le_bytes();
                                    
                                    info!("[{}] Using server GUID: 0x{:08x}", self.addr, server_guid);
                                    
                                    let response = vec![
                                        0x00, 0x00, // Opcode 0x0000
                                        client_version[0], client_version[1], // Mirror version
                                        client_build[0], client_build[1], // Mirror build
                                        client_field1[0], client_field1[1], // Mirror field
                                        guid_bytes[0], guid_bytes[1], guid_bytes[2], guid_bytes[3], // Server GUID (timestamp-based)
                                        client_field2[0], client_field2[1], // Mirror field
                                        client_status[0], client_status[1], client_status[2], client_status[3], // Mirror client status
                                        client_field3[0], client_field3[1], client_field3[2], client_field3[3], // Mirror field
                                        client_field4[0], client_field4[1], client_field4[2], client_field4[3], // Mirror field EXACTLY
                                    ];
                                    
                                    info!("[{}] Response payload ({} bytes): {}", self.addr, response.len(), hex::encode(&response));
                                    
                                    // Add a small delay (official server has ~20ms delay)
                                    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
                                    
                                    if let Ok(encrypted) = self.handler.encrypt_packet(&response) {
                                        info!("[{}] Encrypted packet breakdown:", self.addr);
                                        info!("[{}]   Total length: {} bytes", self.addr, encrypted.len());
                                        info!("[{}]   Full hex: {}", self.addr, hex::encode(&encrypted));
                                        
                                        // Parse and display structure
                                        if encrypted.len() >= 8 {
                                            info!("[{}]   Magic: {:02x} {:02x}", self.addr, encrypted[0], encrypted[1]);
                                            info!("[{}]   Varint size: {}", self.addr, encrypted[2]);
                                            if encrypted[2] == 1 {
                                                info!("[{}]   Payload length: {} (0x{:02x})", self.addr, encrypted[3], encrypted[3]);
                                                info!("[{}]   Opcode: 0x{:02x}", self.addr, encrypted[4]);
                                                if encrypted.len() > 7 {
                                                    info!("[{}]   Flags: 0x{:02x} 0x{:02x} 0x{:02x}", self.addr, encrypted[5], encrypted[6], encrypted[7]);
                                                }
                                                if encrypted.len() > 8 {
                                                    let enc_data = &encrypted[8..];
                                                    info!("[{}]   Encrypted data: {} bytes", self.addr, enc_data.len());
                                                    info!("[{}]   First 32 bytes: {}", self.addr, hex::encode(&enc_data[..enc_data.len().min(32)]));
                                                }
                                            }
                                        }
                                        
                                        if let Err(e) = self.stream.write_all(&encrypted).await {
                                            error!("[{}] Failed to send 0x0000 response: {}", self.addr, e);
                                        } else {
                                            let _ = self.stream.flush().await;
                                            info!("[{}] ✓ Sent 0x0000 response successfully", self.addr);
                                            info!("[{}] Initial handshake complete - login should now work", self.addr);
                                        }
                                    } else {
                                        error!("[{}] Failed to encrypt 0x0000 response", self.addr);
                                        return Ok(());
                                    }
                                }
                                0x2EE2 => {
                                    info!(
                                        "[{}] 🎮 ReqLogin (0x2EE2) - LOGIN REQUEST!",
                                        self.addr
                                    );
                                    info!(
                                        "[{}] Login packet payload: {} bytes",
                                        self.addr,
                                        decrypted.len()
                                    );
                                    
                                    // Call login handler
                                    match handlers::handle_req_login(&decrypted).await {
                                        Ok(response) => {
                                            info!("[{}] Login handler returned success response", self.addr);
                                            
                                            // Encrypt and send response
                                            if let Ok(encrypted) = self.handler.encrypt_packet(&response) {
                                                if let Err(e) = self.stream.write_all(&encrypted).await {
                                                    error!("[{}] Failed to send AckLogin: {}", self.addr, e);
                                                } else {
                                                    let _ = self.stream.flush().await;
                                                    info!("[{}] ✅ Sent AckLogin (0x30D5) successfully!", self.addr);
                                                }
                                            } else {
                                                error!("[{}] Failed to encrypt AckLogin response", self.addr);
                                            }
                                        }
                                        Err(e) => {
                                            error!("[{}] Login handler failed: {}", self.addr, e);
                                        }
                                    }
                                }
                                _ if game_opcode >= 0x1000 => {
                                    info!(
                                        "[{}] Game message opcode in expected range (>= 0x1000)",
                                        self.addr
                                    );
                                }
                                _ => {
                                    info!(
                                        "[{}] Game message opcode unexpected: 0x{:04x}",
                                        self.addr, game_opcode
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("[{}] Decryption failed: {}", self.addr, e);
                    }
                }
            }

            _ => {
                warn!("[{}] Unhandled opcode: 0x{:02x}", self.addr, opcode);
            }
        }

        Ok(())
    }
}

/// Handle a single client connection
async fn handle_client(
    socket: TcpStream,
    addr: SocketAddr,
    crypto: Arc<ProudNetCrypto>,
) -> Result<()> {
    let mut client = ClientConnection::new(socket, addr, crypto);
    client.handle().await
}

/// Setup database connection
async fn setup_database() -> Result<sqlx::Pool<sqlx::Sqlite>> {
    // TODO: Implement database initialization
    // - Read connection string from config
    // - Run migrations
    // - Return connection pool
    unimplemented!("Database setup not yet implemented")
}
