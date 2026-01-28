//! Test TCP server for ProudNet protocol testing
//!
//! This server accepts RO2 client connections and logs all protocol messages.
//! Used for:
//! 1. Testing encryption handshake with real client
//! 2. Capturing and decrypting 0x25 packets in real-time
//! 3. Extracting game message opcodes (0x1001+)
//!
//! Usage:
//! ```bash
//! cargo run --bin test_server
//! # In RO2 client, connect to localhost:7101
//! ```

use anyhow::Result;
use ro2_common::packet::framing::PacketFrame;
use ro2_common::protocol::{ProudNetHandler, ProudNetSettings};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Connection state for a single client
struct ClientConnection {
    stream: TcpStream,
    addr: SocketAddr,
    handler: ProudNetHandler,
    buffer: Vec<u8>,
}

impl ClientConnection {
    fn new(stream: TcpStream, addr: SocketAddr) -> Self {
        println!("\n[CONNECT] New client: {}", addr);
        
        let settings = ProudNetSettings::default();
        println!("[SETTINGS] Using default ProudNet settings:");
        println!("  - AES key: {} bits", settings.aes_key_bits);
        println!("  - Fast encrypt key: {} bits", settings.fast_encrypt_key_bits);
        println!("  - Version: 0x{:08x}", settings.version);
        
        Self {
            stream,
            addr,
            handler: ProudNetHandler::new(addr),
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
                    println!("\n[DISCONNECT] Client closed connection: {}", self.addr);
                    return Ok(());
                }
                Ok(n) => n,
                Err(e) => {
                    eprintln!("\n[ERROR] Read error from {}: {}", self.addr, e);
                    return Err(e.into());
                }
            };

            // Add to buffer
            self.buffer.extend_from_slice(&read_buf[..n]);
            println!("\n[RECV] {} bytes from {} (buffer: {} bytes)", n, self.addr, self.buffer.len());

            // Try to parse packets
            self.process_buffer().await?;
        }
    }

    /// Process buffered data and parse packets
    async fn process_buffer(&mut self) -> Result<()> {
        loop {
            // Check for policy request (starts with '<')
            if self.buffer.starts_with(b"<policy-file-request/>") {
                println!("\n[0x2F] Flash policy request detected");
                self.buffer.drain(..23); // Remove policy request
                
                // Send XML policy (no ProudNet framing)
                if let Some(response) = self.handler.handle(0x2F, &[])? {
                    println!("[0x2F] Sending XML policy ({} bytes, NO framing)", response.len());
                    self.stream.write_all(&response).await?;
                    self.stream.flush().await?;
                    
                    // Now send 0x04 encryption handshake
                    println!("\n[0x04] Sending encryption handshake");
                    let handshake = self.handler.build_encryption_handshake()?;
                    self.hexdump("0x04 packet", &handshake);
                    self.stream.write_all(&handshake).await?;
                    self.stream.flush().await?;
                }
                continue;
            }

            // Try to parse ProudNet packet
            if self.buffer.len() < 4 {
                // Need at least magic + size byte
                break;
            }

            // Check for ProudNet magic
            if &self.buffer[0..2] != &[0x13, 0x57] {
                println!("\n[ERROR] Invalid packet magic: {:02x} {:02x}", 
                         self.buffer[0], self.buffer[1]);
                println!("[ERROR] Buffer content: {}", hex::encode(&self.buffer[..self.buffer.len().min(64)]));
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
                        eprintln!("[ERROR] Packet parse error: {}", e);
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
        
        println!("\n[PACKET] Opcode: 0x{:02x}, Size: {} bytes", opcode, packet.payload.len());
        self.hexdump(&format!("Opcode 0x{:02x}", opcode), &packet.payload);

        // Handle based on opcode
        match opcode {
            0x05 => {
                println!("[0x05] Encryption response - decrypting AES session key");
                if let Some(response) = self.handler.handle(0x05, &packet.payload)? {
                    println!("[0x06] Sending encryption ready acknowledgment");
                    self.stream.write_all(&response).await?;
                    self.stream.flush().await?;
                }
            }
            
            0x07 => {
                println!("[0x07] Version check");
                if let Some(response) = self.handler.handle(0x07, &packet.payload)? {
                    println!("[0x0A] Sending connection success (session ID: {})", 
                             self.handler.session_id().unwrap_or(0));
                    self.hexdump("0x0A packet", &response);
                    self.stream.write_all(&response).await?;
                    self.stream.flush().await?;
                }
            }
            
            0x1B => {
                println!("[0x1B] Heartbeat request");
                if let Some(response) = self.handler.handle(0x1B, &packet.payload)? {
                    println!("[0x1D] Sending heartbeat acknowledgment");
                    self.stream.write_all(&response).await?;
                    self.stream.flush().await?;
                }
            }
            
            0x25 | 0x26 => {
                println!("[0x{:02x}] ENCRYPTED PACKET - attempting decryption", opcode);
                
                if !self.handler.is_encryption_ready() {
                    println!("[WARNING] Encryption not ready yet, cannot decrypt");
                    return Ok(());
                }
                
                // Decrypt the packet
                match self.handler.decrypt_packet(&packet.payload) {
                    Ok(decrypted) => {
                        println!("[SUCCESS] Decrypted {} bytes!", decrypted.len());
                        self.hexdump("DECRYPTED DATA", &decrypted);
                        
                        // Check if it's a game message (opcode >= 0x1000)
                        if decrypted.len() >= 2 {
                            let game_opcode = u16::from_le_bytes([decrypted[0], decrypted[1]]);
                            println!("\n!!! GAME MESSAGE OPCODE: 0x{:04x} !!!", game_opcode);
                            
                            if game_opcode >= 0x1000 {
                                println!("!!! THIS IS A GAME MESSAGE (0x1000+) !!!");
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[ERROR] Decryption failed: {}", e);
                    }
                }
            }
            
            _ => {
                println!("[INFO] Unhandled opcode: 0x{:02x}", opcode);
            }
        }

        Ok(())
    }

    /// Print hexdump of data
    fn hexdump(&self, label: &str, data: &[u8]) {
        println!("[HEXDUMP] {} ({} bytes):", label, data.len());
        
        // Show first 256 bytes max
        let display_len = data.len().min(256);
        
        for (i, chunk) in data[..display_len].chunks(16).enumerate() {
            print!("  {:04x}  ", i * 16);
            
            // Hex
            for (j, byte) in chunk.iter().enumerate() {
                print!("{:02x} ", byte);
                if j == 7 { print!(" "); }
            }
            
            // Padding
            for _ in chunk.len()..16 {
                print!("   ");
                if chunk.len() <= 8 { print!(" "); }
            }
            
            // ASCII
            print!(" |");
            for byte in chunk {
                let c = if *byte >= 32 && *byte < 127 {
                    *byte as char
                } else {
                    '.'
                };
                print!("{}", c);
            }
            println!("|");
        }
        
        if data.len() > display_len {
            println!("  ... ({} more bytes)", data.len() - display_len);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("==============================================");
    println!("   RO2 ProudNet Protocol Test Server");
    println!("==============================================");
    println!();
    println!("This server will:");
    println!("  1. Accept RO2 client connections");
    println!("  2. Perform ProudNet encryption handshake");
    println!("  3. Decrypt 0x25/0x26 encrypted packets");
    println!("  4. Extract game message opcodes");
    println!();
    
    let addr = "0.0.0.0:7101";
    let listener = TcpListener::bind(addr).await?;
    
    println!("Server listening on: {}", addr);
    println!();
    println!("Configure RO2 client to connect to:");
    println!("  - localhost:7101 (if on same machine)");
    println!("  - {}:7101 (if on different machine)", 
             local_ip_address::local_ip().unwrap_or("0.0.0.0".parse().unwrap()));
    println!();
    println!("Waiting for connections...");
    println!("==============================================");

    loop {
        let (stream, addr) = listener.accept().await?;
        
        // Spawn a task for this client
        tokio::spawn(async move {
            let mut client = ClientConnection::new(stream, addr);
            
            if let Err(e) = client.handle().await {
                eprintln!("\n[ERROR] Client {} error: {}", addr, e);
            }
        });
    }
}
