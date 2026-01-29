//! System message handler (0x1001)
//!
//! Handles NfyServerTimeToLoginPC notification messages.
//!
//! Based on reverse engineering of HandleGamePacket_0x1001_SystemMessage @ 0x006a60a0:
//! - Validates game state (must be in lobby or in-game)
//! - Checks proximity to other players
//! - Uses localization system for messages
//! - Displays message in UI/chat
//! - Can trigger network connection creation

use async_trait::async_trait;
use ro2_common::Result;
use ro2_common::protocol::handler::{GameContext, GameMessageHandler};
use tracing::{debug, info};

/// Handler for system messages/notifications (0x1001)
///
/// This is a notification-type message (Nfy prefix) that the server
/// sends to clients to display system messages, alerts, and notifications.
///
/// Client handler @ 0x006a60a0:
/// ```c
/// void HandleGamePacket_0x1001_SystemMessage(
///     int packet_id,         // 0x1001
///     wchar_t* message_text, // Wide string message content
///     int* context           // Game state context
/// )
/// ```
pub struct SystemMessageHandler;

impl SystemMessageHandler {
    /// Create a new SystemMessageHandler
    pub fn new() -> Self {
        Self
    }
}

impl Default for SystemMessageHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GameMessageHandler for SystemMessageHandler {
    async fn handle(
        &self,
        packet_id: u32,
        data: &[u8],
        context: &mut GameContext,
    ) -> Result<Option<Vec<u8>>> {
        // Verify packet ID matches expected opcode
        if packet_id != 0x1001 {
            return Err(anyhow::anyhow!(
                "SystemMessageHandler received wrong opcode: 0x{:04x}",
                packet_id
            ));
        }

        // Check game state is active (lobby or in-game)
        // Mirrors IsGameStateActive check from 0x006a60a0
        if !context.is_game_state_active() {
            debug!(
                "SystemMessage rejected: game state not active (state: {}, session: {})",
                context.game_state, context.session_id
            );
            return Ok(None);
        }

        // Parse message text from packet data
        // Client expects wide string (UTF-16), we use UTF-8
        let message = match parse_message_text(data) {
            Ok(msg) => msg,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to parse system message: {}", e));
            }
        };

        info!(
            "System message received (session: {}): {}",
            context.session_id, message
        );

        // TODO: Implement full handler logic from 0x006a60a0:
        // 1. Query nearby players (GetPlayerList + proximity check)
        // 2. Use localization system (LocalizationManager_GetString)
        // 3. Display message in UI (DisplaySystemMessage)
        // 4. Create network connection if needed (CreateGameNetworkConnection)

        // For now, we just log the message
        // The server would broadcast this to relevant clients

        // System messages are notifications - no response needed
        Ok(None)
    }

    fn opcode(&self) -> u32 {
        0x1001
    }

    fn name(&self) -> &'static str {
        "SystemMessageHandler"
    }
}

/// Parse message text from packet data
///
/// In the client, messages are wide strings (UTF-16).
/// For the server, we'll use UTF-8 encoded strings.
///
/// Packet format (tentative):
/// - u16: message_length (number of characters)
/// - u8[]: message_text (UTF-8 encoded)
fn parse_message_text(data: &[u8]) -> Result<String> {
    if data.len() < 2 {
        return Err(anyhow::anyhow!("Packet too short for message length"));
    }

    // Read message length (u16 little-endian)
    let length = u16::from_le_bytes([data[0], data[1]]) as usize;

    if data.len() < 2 + length {
        return Err(anyhow::anyhow!(
            "Packet too short for message text (expected {} bytes, got {})",
            2 + length,
            data.len()
        ));
    }

    // Parse UTF-8 string
    let message = String::from_utf8(data[2..2 + length].to_vec())
        .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in message: {}", e))?;

    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_message_text() {
        let message = "Hello, world!";
        let mut data = vec![];
        data.extend_from_slice(&(message.len() as u16).to_le_bytes());
        data.extend_from_slice(message.as_bytes());

        let parsed = parse_message_text(&data).unwrap();
        assert_eq!(parsed, message);
    }

    #[test]
    fn test_parse_message_text_empty() {
        let data = vec![0, 0]; // Length = 0
        let parsed = parse_message_text(&data).unwrap();
        assert_eq!(parsed, "");
    }

    #[test]
    fn test_parse_message_text_too_short() {
        let data = vec![5, 0]; // Length = 5, but no data
        let result = parse_message_text(&data);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_system_message_handler() {
        let handler = SystemMessageHandler::new();

        // Create test context in active game state
        let mut context = GameContext::new(123, "127.0.0.1:8080".to_string());
        context.game_state = 2; // In-game

        // Create test message packet
        let message = "Test system message";
        let mut data = vec![];
        data.extend_from_slice(&(message.len() as u16).to_le_bytes());
        data.extend_from_slice(message.as_bytes());

        let response = handler.handle(0x1001, &data, &mut context).await;

        assert!(response.is_ok());
        assert_eq!(response.unwrap(), None); // No response for notifications
    }

    #[tokio::test]
    async fn test_system_message_handler_wrong_opcode() {
        let handler = SystemMessageHandler::new();
        let mut context = GameContext::new(123, "127.0.0.1:8080".to_string());
        context.game_state = 2;

        let result = handler.handle(0x1002, &[], &mut context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_system_message_handler_inactive_state() {
        let handler = SystemMessageHandler::new();
        let mut context = GameContext::new(123, "127.0.0.1:8080".to_string());
        context.game_state = 0; // Disconnected

        let message = "Test";
        let mut data = vec![];
        data.extend_from_slice(&(message.len() as u16).to_le_bytes());
        data.extend_from_slice(message.as_bytes());

        let response = handler.handle(0x1001, &data, &mut context).await;

        // Should succeed but return None (message rejected)
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), None);
    }
}
