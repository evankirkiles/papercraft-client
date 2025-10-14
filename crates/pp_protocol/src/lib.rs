use pp_core::CommandType;
use serde::{Deserialize, Serialize};

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Join a document session
    Join { doc_id: String },

    /// Send a command from the user interacting on the client
    Command { command: CommandType, rollback: bool },

    /// Request the current full state
    RequestSync,
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Successfully joined the session with initial state
    Joined { doc_id: String, state: Vec<u8>, version: u64, client_count: usize },

    /// An operation from another client that should be applied
    Command { client_id: String, command: CommandType, rollback: bool, version: u64 },

    /// Full state sync response
    StateSync { state: Vec<u8>, version: u64 },

    /// Client joined the session
    ClientJoined { client_id: String, client_count: usize },

    /// Client left the session
    ClientLeft { client_id: String, client_count: usize },
}
