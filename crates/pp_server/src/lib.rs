pub mod session;
pub mod store;

use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use session::DocumentSession;
use std::{collections::HashMap, sync::Arc, time::Duration};
use store::DocumentStore;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::info;

/// Main server that manages all document sessions
pub struct Server {
    /// Storage backend for persisting documents
    store: Arc<dyn DocumentStore>,
    /// Active document sessions
    sessions: Arc<RwLock<HashMap<String, Arc<DocumentSession>>>>,
    /// How often to persist documents to storage
    persistence_interval: Duration,
}

impl Server {
    /// Create a new server with the given storage backend
    pub fn new(store: impl DocumentStore + 'static, persistence_interval: Duration) -> Self {
        Self {
            store: Arc::new(store),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            persistence_interval,
        }
    }

    /// Start the server on the given address
    pub async fn run(self, addr: &str) -> anyhow::Result<()> {
        let server = Arc::new(self);

        // Start the periodic persistence task
        tokio::spawn(persistence_task(Arc::clone(&server.sessions), server.persistence_interval));

        // Build our application router
        let app = Router::new()
            .route("/health", get(health_check))
            .route("/documents/{doc_id}", get(websocket_handler))
            .layer(CorsLayer::permissive())
            .with_state(Arc::clone(&server));

        info!("Server listening on {}", addr);

        // Run the server
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    /// Get or create a document session
    async fn get_or_create_session(&self, doc_id: &str) -> anyhow::Result<Arc<DocumentSession>> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get(doc_id) {
            return Ok(Arc::clone(session));
        }

        // Create new session
        info!("Creating new session for document: {}", doc_id);
        let session =
            Arc::new(DocumentSession::new(doc_id.to_string(), Arc::clone(&self.store)).await?);
        sessions.insert(doc_id.to_string(), Arc::clone(&session));

        Ok(session)
    }
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// WebSocket handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(doc_id): Path<String>,
    State(server): State<Arc<Server>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, doc_id, server))
}

/// Handle a WebSocket connection
async fn handle_socket(socket: WebSocket, doc_id: String, server: Arc<Server>) {
    // Generate a unique client ID
    let client_id = uuid::Uuid::new_v4().to_string();

    // Get or create the document session
    let session = match server.get_or_create_session(&doc_id).await {
        Ok(session) => session,
        Err(e) => {
            tracing::error!("Failed to get or create session: {:?}", e);
            return;
        }
    };

    // Handle the client
    if let Err(e) = session.handle_client(socket, client_id).await {
        tracing::error!("Client connection error: {:?}", e);
    }
}

/// Background task that periodically persists all active documents
async fn persistence_task(
    sessions: Arc<RwLock<HashMap<String, Arc<DocumentSession>>>>,
    interval: Duration,
) {
    let mut interval_timer = tokio::time::interval(interval);

    loop {
        interval_timer.tick().await;

        let sessions_snapshot = sessions.read().await;
        let doc_ids: Vec<_> = sessions_snapshot.keys().cloned().collect();
        drop(sessions_snapshot);

        for doc_id in doc_ids {
            let sessions = sessions.read().await;
            if let Some(session) = sessions.get(&doc_id) {
                let session = Arc::clone(session);
                drop(sessions);

                // Only persist if there are active clients
                if session.client_count().await > 0 {
                    if let Err(e) = session.persist().await {
                        tracing::error!("Failed to persist document {}: {:?}", doc_id, e);
                    }
                }
            }
        }

        // Clean up sessions with no clients
        let mut sessions = sessions.write().await;
        sessions.retain(|doc_id, session| {
            let count = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async { session.client_count().await })
            });
            if count == 0 {
                info!("Cleaning up session for document: {}", doc_id);
                false
            } else {
                true
            }
        });
    }
}
