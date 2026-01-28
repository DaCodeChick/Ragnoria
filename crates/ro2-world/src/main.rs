//! RO2 World Server
//!
//! Handles game world simulation on port 7401
//! (Minimal implementation for proof of concept)

mod handlers;

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, warn, error};
use std::net::SocketAddr;

const WORLD_PORT: u16 = 7401;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();
    
    info!("Starting RO2 World Server v{}", env!("CARGO_PKG_VERSION"));
    
    // Bind to world port
    let addr = SocketAddr::from(([0, 0, 0, 0], WORLD_PORT));
    let listener = TcpListener::bind(addr).await?;
    
    info!("World server listening on {}", addr);
    info!("NOTE: World server is minimal PoC implementation");
    
    // Accept connections
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                info!("New connection from {}", addr);
                
                tokio::spawn(async move {
                    if let Err(e) = handle_client(socket, addr).await {
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

/// Handle a single client connection
async fn handle_client(mut socket: TcpStream, addr: SocketAddr) -> Result<()> {
    info!("Handling client {}", addr);
    
    let mut buffer = vec![0u8; 4096];
    
    loop {
        let n = socket.read(&mut buffer).await?;
        
        if n == 0 {
            info!("Client {} disconnected", addr);
            break;
        }
        
        info!("Received {} bytes from {}", n, addr);
        
        // TODO: Implement game world logic
        // For now, just echo to keep connection alive
        socket.write_all(&buffer[..n]).await?;
    }
    
    Ok(())
}
