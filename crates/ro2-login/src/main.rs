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
                                        "[{}] Game message 0x0000: Initial handshake? Data: {}",
                                        self.addr,
                                        hex::encode(&decrypted[2..decrypted.len().min(18)])
                                    );
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
