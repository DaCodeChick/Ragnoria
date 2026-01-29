//! Lobby message handlers

use anyhow::Result;

/// Handle ReqLoginChannel message
pub async fn handle_req_login_channel(_data: &[u8]) -> Result<Vec<u8>> {
    // TODO: Implement lobby login handler
    // 1. Parse session key from data
    // 2. Validate session key against database
    // 3. Query character list for account
    // 4. Return AnsLoginChannel with character list

    unimplemented!("ReqLoginChannel handler not yet implemented")
}

/// Handle ReqChannelList message
pub async fn handle_req_channel_list(_data: &[u8]) -> Result<Vec<u8>> {
    // TODO: Implement channel list handler
    // 1. Query available game channels
    // 2. Return AckChannelListInGame with channel info

    unimplemented!("ReqChannelList handler not yet implemented")
}

/// Handle ReqChannelMove message
pub async fn handle_req_channel_move(_data: &[u8]) -> Result<Vec<u8>> {
    // TODO: Implement channel move handler
    // 1. Parse channel ID from data
    // 2. Validate channel exists and has capacity
    // 3. Return AnsChannelMove with world server address

    unimplemented!("ReqChannelMove handler not yet implemented")
}
