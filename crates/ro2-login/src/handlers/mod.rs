//! Login message handlers

use anyhow::Result;

/// Handle ReqLogin message
pub async fn handle_req_login(_data: &[u8]) -> Result<Vec<u8>> {
    // TODO: Implement login request handler
    // 1. Parse username and password from data
    // 2. Query database for account
    // 3. Validate password hash
    // 4. Generate session key
    // 5. Return AnsLogin response
    
    unimplemented!("ReqLogin handler not yet implemented")
}

/// Handle ReqServerStatus message
pub async fn handle_req_server_status(_data: &[u8]) -> Result<Vec<u8>> {
    // TODO: Implement server status handler
    // 1. Query available lobby/world servers
    // 2. Return AckServerStatus with server list
    
    unimplemented!("ReqServerStatus handler not yet implemented")
}
