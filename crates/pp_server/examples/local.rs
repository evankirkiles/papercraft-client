use pp_server::{store::FilesystemStore, Server};
use std::{path::PathBuf, time::Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pp_server=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create storage directory
    let storage_path = PathBuf::from("./documents");
    tokio::fs::create_dir_all(&storage_path).await?;

    // Create filesystem store
    let store = FilesystemStore::new(storage_path);

    // Create server with 30-second persistence interval
    let server = Server::new(store, Duration::from_secs(30));

    // Run server on localhost:8080
    println!("Starting pp_server on http://localhost:8080");
    println!("Documents will be saved to ./documents/");
    println!("WebSocket endpoint: ws://localhost:8080/documents/:doc_id");
    println!("Health check: http://localhost:8080/health");

    server.run("127.0.0.1:8080").await?;

    Ok(())
}
