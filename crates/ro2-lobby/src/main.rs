//! RO2 Lobby Server
//!
//! Handles channel selection and character management on port 7201

mod handlers;

use anyhow::Result;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info};

const LOBBY_PORT: u16 = 7201;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("Starting RO2 Lobby Server v{}", env!("CARGO_PKG_VERSION"));

    // Bind to lobby port
    let addr = SocketAddr::from(([0, 0, 0, 0], LOBBY_PORT));
    let listener = TcpListener::bind(addr).await?;

    info!("Lobby server listening on {}", addr);

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

        // TODO: Parse packet and route to appropriate handler
        socket.write_all(&buffer[..n]).await?;
    }

    Ok(())
}
