//! RO2 Login Server
//!
//! Handles client authentication on port 7101

mod handlers;

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, warn, error};
use std::net::SocketAddr;

const LOGIN_PORT: u16 = 7101;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();
    
    info!("Starting RO2 Login Server v{}", env!("CARGO_PKG_VERSION"));
    
    // TODO: Initialize database connection
    // let db = setup_database().await?;
    
    // Bind to login port
    let addr = SocketAddr::from(([0, 0, 0, 0], LOGIN_PORT));
    let listener = TcpListener::bind(addr).await?;
    
    info!("Login server listening on {}", addr);
    
    // Accept connections
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                info!("New connection from {}", addr);
                
                // Spawn a task to handle this client
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
        // Read data from client
        let n = socket.read(&mut buffer).await?;
        
        if n == 0 {
            info!("Client {} disconnected", addr);
            break;
        }
        
        info!("Received {} bytes from {}", n, addr);
        
        // TODO: Parse packet and route to appropriate handler
        // For now, just echo back for testing
        socket.write_all(&buffer[..n]).await?;
    }
    
    Ok(())
}

/// Setup database connection
async fn setup_database() -> Result<sqlx::Pool<sqlx::Sqlite>> {
    // TODO: Implement database initialization
    // - Read connection string from config
    // - Run migrations
    // - Return connection pool
    unimplemented!("Database setup not yet implemented")
}
