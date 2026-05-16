use anyhow::Result;
use async_trait::async_trait;

use super::{Document, DocumentChunk, EmbeddingVector, RetrievedChunk};

/// A single message in a conversation.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[async_trait]
pub trait Embedder: Send + Sync {
    async fn embed_texts(&self, texts: &[String]) -> Result<Vec<EmbeddingVector>>;
    async fn embed_query(&self, query: &str) -> Result<EmbeddingVector>;
}

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn create_collection_if_not_exists(&self, vector_size: u64) -> Result<()>;
    async fn upsert_chunks(
        &self,
        chunks: &[DocumentChunk],
        vectors: &[EmbeddingVector],
    ) -> Result<()>;
    async fn search(
        &self,
        query_vector: &EmbeddingVector,
        top_k: usize,
        document_id: Option<&str>,
    ) -> Result<Vec<RetrievedChunk>>;
}

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn generate_answer(&self, system_prompt: &str, user_prompt: &str) -> Result<String>;
    async fn generate_answer_with_history(
        &self,
        system_prompt: &str,
        history: &[ChatMessage],
        user_prompt: &str,
    ) -> Result<String>;
}

pub trait DocumentLoader: Send + Sync {
    fn load(&self, file_path: &std::path::Path, file_name: &str) -> Result<Document>;
}
