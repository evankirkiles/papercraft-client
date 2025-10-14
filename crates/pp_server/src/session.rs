use crate::protocol::{ClientMessage, ServerMessage};
use crate::store::DocumentStore;
use anyhow::Result;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use futures::{stream::StreamExt, SinkExt};
use std::io::Cursor;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info, warn};

use pp_core::{Command, State};
use pp_save::save::Saveable;
use pp_save::{load::Loadable, SaveFile};

#[derive(Debug, Copy, Clone, Error)]
pub enum ClientConnectError {
    #[error("An unknown error occurred.")]
    Unknown,
    #[error("Failed to send initial document state")]
    BadInitialSave,
    #[error("Failed to send ACK message")]
    BadJoin,
}

/// A document session manages all clients connected to a single document
pub struct DocumentSession {
    doc_id: String,
    /// Current state of the document
    state: Arc<RwLock<State>>,
    /// Version counter for optimistic locking
    version: Arc<RwLock<u64>>,
    /// Broadcast channel for sending operations to all clients
    tx: broadcast::Sender<(String, Utf8Bytes)>,
    /// Storage backend
    store: Arc<dyn DocumentStore>,
    /// Track number of connected clients
    client_count: Arc<RwLock<usize>>,
}

impl DocumentSession {
    /// Create a new session for a document
    pub async fn new(doc_id: String, store: Arc<dyn DocumentStore>) -> Result<Self> {
        // Try to load existing document, or create new one
        let state = if store.exists(&doc_id).await? {
            info!("Loading existing document: {}", doc_id);
            let bytes = store.load(&doc_id).await?;
            let save_file = SaveFile::from_reader(Cursor::new(bytes))?;
            State::load(save_file)?
        } else {
            info!("Creating new document: {}", doc_id);
            State::default()
        };

        let (tx, _rx) = broadcast::channel(100);

        Ok(Self {
            doc_id,
            state: Arc::new(RwLock::new(state)),
            version: Arc::new(RwLock::new(0)),
            tx,
            store,
            client_count: Arc::new(RwLock::new(0)),
        })
    }

    /// Handle a new client connection
    pub async fn handle_client(
        &self,
        socket: WebSocket,
        client_id: String,
    ) -> Result<(), ClientConnectError> {
        info!("Client {} connecting to document {}", client_id, self.doc_id);

        // Increment client count
        {
            let mut count = self.client_count.write().await;
            *count += 1;
        }

        // Split the socket into sender and receiver
        let (mut ws_sender, mut ws_receiver) = socket.split();
        // Subscribe to broadcast channel for operations from other clients
        let mut op_receiver = self.tx.subscribe();

        // Send initial state to the client
        let state = self.state.read().await;
        let version = *self.version.read().await;
        let client_count = *self.client_count.read().await;
        let initial_message = state
            .save()
            .and_then(|save_file| save_file.to_binary())
            .map(|state| {
                serde_json::to_string(&ServerMessage::Joined {
                    doc_id: self.doc_id.clone(),
                    state,
                    version,
                    client_count,
                })
                .unwrap()
                .into()
            })
            .map_err(|_| ClientConnectError::BadInitialSave)?;
        drop(state);

        // Send the join message with the initial state to the new client
        ws_sender
            .send(Message::Text(initial_message))
            .await
            .map_err(|_| ClientConnectError::BadJoin)?;

        // Notify other clients that someone joined
        let client_id_clone = client_id.clone();
        let _ = self.tx.send((
            "server".to_string(),
            serde_json::to_string(&ServerMessage::ClientJoined {
                client_id: client_id_clone,
                client_count,
            })
            .unwrap()
            .into(),
        ));

        // Clone necessary Arc references for the tasks
        let state = Arc::clone(&self.state);
        let version_arc = Arc::clone(&self.version);
        let client_id_clone = client_id.clone();
        let tx = self.tx.clone();

        // Spawn task to handle incoming messages from this client
        let incoming_task = tokio::spawn(async move {
            while let Some(msg) = ws_receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(ClientMessage::Command { command }) => {
                                // Apply the command to the state
                                let mut state = state.write().await;
                                if let Err(e) = command.execute(&mut state) {
                                    warn!("Failed to apply operation: {:?}", e);
                                    continue;
                                }

                                // Increment the state version
                                let mut version = version_arc.write().await;
                                *version += 1;
                                let new_version = *version;
                                drop(version);
                                drop(state);

                                // Now broadcast the command to other clients.
                                // In the future, we will have more complex logic
                                // to handle clashing commands / rollback.
                                let _ = tx.send((
                                    client_id_clone.clone(),
                                    serde_json::to_string(&ServerMessage::Command {
                                        client_id: client_id_clone.clone(),
                                        command,
                                        version: new_version,
                                    })
                                    .unwrap()
                                    .into(),
                                ));
                            }
                            Ok(ClientMessage::RequestSync) => {
                                // Client requesting full state sync
                                // This is handled by sending a StateSync message
                                // We'll implement this later if needed
                                info!("Client {} requested sync", client_id_clone);
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        warn!("WebSocket error from client {}: {:?}", client_id_clone, e);
                        break;
                    }
                    _ => {
                        // Binary messages not supported
                    }
                }
            }
        });

        // Spawn task to forward operations from other clients to this client
        let client_id_clone = client_id.clone();
        let outgoing_task = tokio::spawn(async move {
            while let Ok((sender_id, msg)) = op_receiver.recv().await {
                // Don't send operations back to the client that sent them
                if sender_id == client_id_clone {
                    continue;
                }

                if let Err(e) = ws_sender.send(Message::Text(msg)).await {
                    error!("Failed to send operation: {:?}", e);
                    break;
                }
            }
        });

        // Wait for either task to complete, then abort the other
        let incoming_abort = incoming_task.abort_handle();
        let outgoing_abort = outgoing_task.abort_handle();
        tokio::select! {
            _ = incoming_task => {
                outgoing_abort.abort();
            },
            _ = outgoing_task => {
                incoming_abort.abort();
            },
        }

        // Decrement client count and get the new count
        let new_client_count = {
            let mut count = self.client_count.write().await;
            *count = count.saturating_sub(1);
            *count
        };

        info!(
            "Client {} disconnected from document {} ({} clients remaining)",
            client_id, self.doc_id, new_client_count
        );

        // Notify other clients that someone left
        let client_id_clone = client_id.clone();
        let _ = self.tx.send((
            "server".to_string(),
            serde_json::to_string(&ServerMessage::ClientLeft {
                client_id: client_id_clone,
                client_count: new_client_count,
            })
            .unwrap()
            .into(),
        ));

        // If this was the last client, persist the document to ensure no data loss
        if new_client_count == 0 {
            if let Err(e) = self.persist().await {
                error!(
                    "Failed to persist document {} on last client disconnect: {:?}",
                    self.doc_id, e
                );
            }
        }

        Ok(())
    }

    /// Persist the current state to storage
    pub async fn persist(&self) -> Result<()> {
        let state = self.state.read().await;
        let save_file = state.save()?;
        let bytes = save_file.to_binary()?;
        self.store.save(&self.doc_id, &bytes).await?;
        info!("Persisted document {} ({} bytes)", self.doc_id, bytes.len());
        Ok(())
    }

    /// Get the current client count
    pub async fn client_count(&self) -> usize {
        *self.client_count.read().await
    }
}
