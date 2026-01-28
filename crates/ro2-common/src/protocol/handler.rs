//! Game message handler infrastructure
//!
//! Based on reverse engineering of Rag2.exe game message handlers.
//! Handlers follow a consistent 3-parameter pattern:
//! - packet_id: Message opcode (0x1001+)
//! - data: Serialized message payload
//! - context: Game state and session context

use crate::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Game context passed to all message handlers
///
/// Mirrors the context parameter from client handlers (param_3 @ 0x006a60a0).
/// Contains game state, session info, and subsystem references.
#[derive(Clone)]
pub struct GameContext {
    /// Session ID for this connection
    pub session_id: u64,
    
    /// Current game state (0=disconnected, 1=lobby, 2=in_game)
    pub game_state: u32,
    
    /// Character ID (if in-game)
    pub character_id: Option<u32>,
    
    /// Account ID
    pub account_id: Option<u32>,
    
    /// Connection metadata
    pub connection_info: ConnectionInfo,
}

/// Connection metadata
#[derive(Clone)]
pub struct ConnectionInfo {
    /// Remote IP address
    pub remote_addr: String,
    
    /// Connection timestamp
    pub connected_at: chrono::DateTime<chrono::Utc>,
    
    /// Last activity timestamp
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

impl GameContext {
    /// Create a new game context for a connection
    pub fn new(session_id: u64, remote_addr: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            session_id,
            game_state: 0, // Disconnected
            character_id: None,
            account_id: None,
            connection_info: ConnectionInfo {
                remote_addr,
                connected_at: now,
                last_activity: now,
            },
        }
    }
    
    /// Check if game state is active (lobby or in-game)
    ///
    /// Mirrors IsGameStateActive check from 0x006a60a0
    pub fn is_game_state_active(&self) -> bool {
        self.game_state == 1 || self.game_state == 2
    }
    
    /// Update last activity timestamp
    pub fn update_activity(&mut self) {
        self.connection_info.last_activity = chrono::Utc::now();
    }
}

/// Trait for game message handlers
///
/// Pattern discovered from HandleGamePacket_0x1001_SystemMessage @ 0x006a60a0:
/// - Handlers validate packet_id matches their expected opcode
/// - Handlers check game_state before processing
/// - Handlers return Result<Option<Vec<u8>>> (Some = response packet, None = no response)
#[async_trait]
pub trait GameMessageHandler: Send + Sync {
    /// Handle a game message
    ///
    /// # Parameters
    /// - `packet_id`: Message opcode (e.g., 0x1001)
    /// - `data`: Serialized message payload
    /// - `context`: Game state and session context
    ///
    /// # Returns
    /// - `Ok(Some(response))`: Handler processed message and has response packet
    /// - `Ok(None)`: Handler processed message but no response needed
    /// - `Err(e)`: Handler failed to process message
    async fn handle(
        &self,
        packet_id: u32,
        data: &[u8],
        context: &mut GameContext,
    ) -> Result<Option<Vec<u8>>>;
    
    /// Get the message opcode this handler handles
    fn opcode(&self) -> u32;
    
    /// Get handler name for logging
    fn name(&self) -> &'static str;
}

/// Type alias for boxed handler
pub type BoxedHandler = Arc<dyn GameMessageHandler>;

/// Handler registry for looking up handlers by opcode
pub struct HandlerRegistry {
    handlers: std::collections::HashMap<u32, BoxedHandler>,
}

impl HandlerRegistry {
    /// Create a new empty handler registry
    pub fn new() -> Self {
        Self {
            handlers: std::collections::HashMap::new(),
        }
    }
    
    /// Register a handler for an opcode
    pub fn register(&mut self, handler: BoxedHandler) {
        let opcode = handler.opcode();
        self.handlers.insert(opcode, handler);
    }
    
    /// Get handler for an opcode
    pub fn get(&self, opcode: u32) -> Option<&BoxedHandler> {
        self.handlers.get(&opcode)
    }
    
    /// Check if handler is registered for opcode
    pub fn has_handler(&self, opcode: u32) -> bool {
        self.handlers.contains_key(&opcode)
    }
    
    /// Get all registered opcodes
    pub fn registered_opcodes(&self) -> Vec<u32> {
        self.handlers.keys().copied().collect()
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestHandler;
    
    #[async_trait]
    impl GameMessageHandler for TestHandler {
        async fn handle(
            &self,
            _packet_id: u32,
            _data: &[u8],
            _context: &mut GameContext,
        ) -> Result<Option<Vec<u8>>> {
            Ok(None)
        }
        
        fn opcode(&self) -> u32 {
            0x1001
        }
        
        fn name(&self) -> &'static str {
            "TestHandler"
        }
    }
    
    #[test]
    fn test_handler_registry() {
        let mut registry = HandlerRegistry::new();
        let handler = Arc::new(TestHandler);
        
        registry.register(handler);
        
        assert!(registry.has_handler(0x1001));
        assert!(!registry.has_handler(0x1002));
        assert_eq!(registry.registered_opcodes(), vec![0x1001]);
    }
    
    #[test]
    fn test_game_context() {
        let ctx = GameContext::new(123, "127.0.0.1:8080".to_string());
        
        assert_eq!(ctx.session_id, 123);
        assert_eq!(ctx.game_state, 0);
        assert!(!ctx.is_game_state_active());
        
        let mut ctx = ctx;
        ctx.game_state = 1;
        assert!(ctx.is_game_state_active());
        
        ctx.game_state = 2;
        assert!(ctx.is_game_state_active());
    }
}
