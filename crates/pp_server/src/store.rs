use async_trait::async_trait;
use std::path::PathBuf;

/// Trait for persisting document state to various storage backends
#[async_trait]
pub trait DocumentStore: Send + Sync {
    /// Load a document from persistent storage
    async fn load(&self, doc_id: &str) -> anyhow::Result<Vec<u8>>;
    /// Save a document to persistent storage
    async fn save(&self, doc_id: &str, data: &[u8]) -> anyhow::Result<()>;
    /// Check if a document exists
    async fn exists(&self, doc_id: &str) -> anyhow::Result<bool>;
}

/// Simple filesystem-based storage implementation
pub struct FilesystemStore {
    root: PathBuf,
}

impl FilesystemStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn document_path(&self, doc_id: &str) -> PathBuf {
        self.root.join(format!("{}.glb", doc_id))
    }
}

#[async_trait]
impl DocumentStore for FilesystemStore {
    async fn load(&self, doc_id: &str) -> anyhow::Result<Vec<u8>> {
        let path = self.document_path(doc_id);
        Ok(tokio::fs::read(path).await?)
    }

    async fn save(&self, doc_id: &str, data: &[u8]) -> anyhow::Result<()> {
        let path = self.document_path(doc_id);

        // Ensure the directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        Ok(tokio::fs::write(path, data).await?)
    }

    async fn exists(&self, doc_id: &str) -> anyhow::Result<bool> {
        let path = self.document_path(doc_id);
        Ok(tokio::fs::try_exists(path).await?)
    }
}
