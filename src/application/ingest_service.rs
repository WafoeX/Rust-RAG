use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

use crate::domain::ports::{Embedder, VectorStore};
use crate::infrastructure::document_loader::load_document_by_extension;
use crate::utils::text_splitter::split_text_to_chunks;

pub struct IngestService {
    embedder: Arc<dyn Embedder>,
    vector_store: Arc<dyn VectorStore>,
    chunk_size: usize,
    chunk_overlap: usize,
}

pub struct IngestResult {
    pub document_id: String,
    pub file_name: String,
    pub chunk_count: usize,
}

impl IngestService {
    pub fn new(
        embedder: Arc<dyn Embedder>,
        vector_store: Arc<dyn VectorStore>,
        chunk_size: usize,
        chunk_overlap: usize,
    ) -> Self {
        Self {
            embedder,
            vector_store,
            chunk_size,
            chunk_overlap,
        }
    }

    pub async fn ingest(&self, file_path: &Path, file_name: &str) -> Result<IngestResult> {
        let document = load_document_by_extension(file_path, file_name)?;
        let document_id = document.id.clone();
        let file_name = document.file_name.clone();

        let chunks = split_text_to_chunks(&document, self.chunk_size, self.chunk_overlap);

        if chunks.is_empty() {
            return Ok(IngestResult {
                document_id,
                file_name,
                chunk_count: 0,
            });
        }

        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let vectors = self.embedder.embed_texts(&texts).await?;
        self.vector_store.upsert_chunks(&chunks, &vectors).await?;

        Ok(IngestResult {
            document_id,
            file_name,
            chunk_count: chunks.len(),
        })
    }
}
