//! Login message handlers

use anyhow::Result;
use tracing::info;

/// Handle ReqLogin (0x2EE2) message
/// 
/// Packet structure (211 bytes total):
/// - Opcode: 2 bytes (0x2EE2)
/// - Payload: 209 bytes (username, password, version, etc.)
/// 
/// Response: AckLogin (0x30D5) - 82 bytes total (2 byte opcode + 80 byte payload)
pub async fn handle_req_login(data: &[u8]) -> Result<Vec<u8>> {
    info!("📧 ReqLogin (0x2EE2) received: {} bytes", data.len());
    info!("   Raw hex (first 64 bytes): {}", hex::encode(&data[..data.len().min(64)]));
    
    // For now, accept any login and return success
    // Real implementation would:
    // 1. Parse username/password from the 209-byte structure
    // 2. Validate credentials against database
    // 3. Generate proper session tokens
    
    // Build AckLogin (0x30D5) response
    // Structure: 2 bytes opcode + 80 bytes payload = 82 bytes total
    let mut response = Vec::new();
    
    // Opcode 0x30D5 (little endian)
    response.extend_from_slice(&[0xD5, 0x30]);
    
    // Result code (4 bytes) - 0 = success
    response.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    
    // Account ID (4 bytes) - dummy value
    response.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
    
    // Session token (16 bytes) - random
    let session_token: [u8; 16] = rand::random();
    response.extend_from_slice(&session_token);
    
    // Remaining payload (56 bytes) - fill with zeros for now
    // This would contain: account flags, character slots, premium status, etc.
    response.extend(vec![0u8; 56]);
    
    info!("✅ Sending AckLogin (0x30D5) - Login SUCCESS");
    info!("   Response: {} bytes", response.len());
    
    Ok(response)
}

/// Handle ReqServerStatus message
pub async fn handle_req_server_status(_data: &[u8]) -> Result<Vec<u8>> {
    // TODO: Implement server status handler
    // 1. Query available lobby/world servers
    // 2. Return AckServerStatus with server list

    unimplemented!("ReqServerStatus handler not yet implemented")
}
