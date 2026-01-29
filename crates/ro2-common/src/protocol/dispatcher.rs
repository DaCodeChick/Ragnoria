//! Message dispatcher for routing game packets to handlers
//!
//! Based on reverse engineering of game message dispatch system.
//! The client uses function pointers to dispatch messages to handlers,
//! we use a HashMap-based registry for flexibility.

use super::handler::{BoxedHandler, GameContext, HandlerRegistry};
use crate::Result;
use tracing::{debug, error, warn};

/// Message dispatcher routes incoming packets to registered handlers
///
/// Architecture mirrors the client's dispatch system:
/// - Packet arrives with opcode (u32)
/// - Dispatcher looks up handler by opcode
/// - Handler processes packet and optionally returns response
///
/// Key differences from client:
/// - Client uses function pointers in a switch/table
/// - Server uses HashMap for dynamic handler registration
/// - Server handlers are async (client handlers are synchronous)
pub struct MessageDispatcher {
    /// Handler registry (opcode -> handler)
    registry: HandlerRegistry,
    
    /// Statistics
    stats: DispatcherStats,
}

/// Dispatcher statistics
#[derive(Debug, Default)]
pub struct DispatcherStats {
    /// Total messages processed
    pub messages_processed: u64,
    
    /// Messages processed successfully
    pub messages_success: u64,
    
    /// Messages that failed processing
    pub messages_failed: u64,
    
    /// Messages with no registered handler
    pub messages_unhandled: u64,
}

impl MessageDispatcher {
    /// Create a new message dispatcher
    pub fn new() -> Self {
        Self {
            registry: HandlerRegistry::new(),
            stats: DispatcherStats::default(),
        }
    }
    
    /// Create a dispatcher with pre-registered handlers
    pub fn with_handlers(handlers: Vec<BoxedHandler>) -> Self {
        let mut dispatcher = Self::new();
        for handler in handlers {
            dispatcher.register_handler(handler);
        }
        dispatcher
    }
    
    /// Register a handler for an opcode
    pub fn register_handler(&mut self, handler: BoxedHandler) {
        let opcode = handler.opcode();
        debug!("Registering handler for opcode 0x{:04x}: {}", opcode, handler.name());
        self.registry.register(handler);
    }
    
    /// Dispatch a message to its handler
    ///
    /// # Parameters
    /// - `packet_id`: Message opcode (e.g., 0x1001)
    /// - `data`: Serialized message payload
    /// - `context`: Game state and session context
    ///
    /// # Returns
    /// - `Ok(Some(response))`: Handler processed message and has response
    /// - `Ok(None)`: Handler processed message but no response needed
    /// - `Err(e)`: Handler failed or no handler registered
    pub async fn dispatch(
        &mut self,
        packet_id: u32,
        data: &[u8],
        context: &mut GameContext,
    ) -> Result<Option<Vec<u8>>> {
        self.stats.messages_processed += 1;
        
        // Look up handler
        let handler = match self.registry.get(packet_id) {
            Some(h) => h,
            None => {
                self.stats.messages_unhandled += 1;
                warn!(
                    "No handler registered for opcode 0x{:04x} (session: {})",
                    packet_id, context.session_id
                );
                return Ok(None);
            }
        };
        
        // Dispatch to handler
        debug!(
            "Dispatching opcode 0x{:04x} to {} (session: {})",
            packet_id,
            handler.name(),
            context.session_id
        );
        
        match handler.handle(packet_id, data, context).await {
            Ok(response) => {
                self.stats.messages_success += 1;
                debug!(
                    "Handler {} completed successfully (session: {})",
                    handler.name(),
                    context.session_id
                );
                Ok(response)
            }
            Err(e) => {
                self.stats.messages_failed += 1;
                error!(
                    "Handler {} failed: {} (session: {})",
                    handler.name(),
                    e,
                    context.session_id
                );
                Err(e)
            }
        }
    }
    
    /// Check if handler is registered for opcode
    pub fn has_handler(&self, opcode: u32) -> bool {
        self.registry.has_handler(opcode)
    }
    
    /// Get list of all registered opcodes
    pub fn registered_opcodes(&self) -> Vec<u32> {
        self.registry.registered_opcodes()
    }
    
    /// Get dispatcher statistics
    pub fn stats(&self) -> &DispatcherStats {
        &self.stats
    }
    
    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = DispatcherStats::default();
    }
}

impl Default for MessageDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::handler::GameMessageHandler;
    use async_trait::async_trait;
    use std::sync::Arc;
    
    struct TestHandler {
        opcode: u32,
        name: &'static str,
    }
    
    #[async_trait]
    impl GameMessageHandler for TestHandler {
        async fn handle(
            &self,
            _packet_id: u32,
            _data: &[u8],
            _context: &mut GameContext,
        ) -> Result<Option<Vec<u8>>> {
            Ok(Some(vec![1, 2, 3, 4]))
        }
        
        fn opcode(&self) -> u32 {
            self.opcode
        }
        
        fn name(&self) -> &'static str {
            self.name
        }
    }
    
    #[tokio::test]
    async fn test_dispatcher_with_handler() {
        let handler = Arc::new(TestHandler {
            opcode: 0x1001,
            name: "TestHandler",
        });
        
        let mut dispatcher = MessageDispatcher::new();
        dispatcher.register_handler(handler);
        
        let mut ctx = GameContext::new(123, "127.0.0.1:8080".to_string());
        let response = dispatcher.dispatch(0x1001, &[1, 2, 3], &mut ctx).await;
        
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), Some(vec![1, 2, 3, 4]));
        assert_eq!(dispatcher.stats().messages_processed, 1);
        assert_eq!(dispatcher.stats().messages_success, 1);
    }
    
    #[tokio::test]
    async fn test_dispatcher_no_handler() {
        let mut dispatcher = MessageDispatcher::new();
        let mut ctx = GameContext::new(123, "127.0.0.1:8080".to_string());
        
        let response = dispatcher.dispatch(0x9999, &[1, 2, 3], &mut ctx).await;
        
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), None);
        assert_eq!(dispatcher.stats().messages_processed, 1);
        assert_eq!(dispatcher.stats().messages_unhandled, 1);
    }
    
    #[test]
    fn test_dispatcher_has_handler() {
        let handler = Arc::new(TestHandler {
            opcode: 0x1001,
            name: "TestHandler",
        });
        
        let mut dispatcher = MessageDispatcher::new();
        dispatcher.register_handler(handler);
        
        assert!(dispatcher.has_handler(0x1001));
        assert!(!dispatcher.has_handler(0x1002));
    }
}
