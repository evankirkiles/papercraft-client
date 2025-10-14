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

/// S3-based storage implementation (behind feature flag)
#[cfg(feature = "s3")]
pub struct S3Store {
    bucket: String,
    client: aws_sdk_s3::Client,
}

#[cfg(feature = "s3")]
impl S3Store {
    pub async fn new(bucket: String) -> Self {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_s3::Client::new(&config);
        Self { bucket, client }
    }

    fn document_key(&self, doc_id: &str) -> String {
        format!("documents/{}.glb", doc_id)
    }
}

#[cfg(feature = "s3")]
#[async_trait]
impl DocumentStore for S3Store {
    async fn load(&self, doc_id: &str) -> anyhow::Result<Vec<u8>> {
        let key = self.document_key(doc_id);
        let output = self.client.get_object().bucket(&self.bucket).key(&key).send().await?;

        let bytes = output.body.collect().await?;
        Ok(bytes.into_bytes().to_vec())
    }

    async fn save(&self, doc_id: &str, data: &[u8]) -> anyhow::Result<()> {
        let key = self.document_key(doc_id);
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(data.to_vec().into())
            .send()
            .await?;

        Ok(())
    }

    async fn exists(&self, doc_id: &str) -> anyhow::Result<bool> {
        let key = self.document_key(doc_id);
        let result = self.client.head_object().bucket(&self.bucket).key(&key).send().await;

        Ok(result.is_ok())
    }
}
