//! World server message handlers (minimal PoC)

use anyhow::Result;

/// Handle player spawn (future implementation)
pub async fn handle_player_spawn(data: &[u8]) -> Result<Vec<u8>> {
    unimplemented!("Player spawn not yet implemented - out of scope for PoC")
}

/// Handle player movement (future implementation)
pub async fn handle_player_movement(data: &[u8]) -> Result<Vec<u8>> {
    unimplemented!("Player movement not yet implemented - out of scope for PoC")
}
