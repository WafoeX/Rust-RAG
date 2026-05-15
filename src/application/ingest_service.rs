use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

use crate::domain::ports::{Embedder, VectorStore};
use crate::infrastructure::document_loader::load_document_by_extension;
use crate::utils::text_splitter::split_text_to_chunks;

const EMBED_BATCH_SIZE: usize = 32;

pub struct IngestService {
    embedder: Arc<dyn Embedder>,
    vector_store: Arc<dyn VectorStore>,
    chunk_size: usize,
    chunk_overlap: usize,
    min_chunk_size: usize,
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
        min_chunk_size: usize,
    ) -> Self {
        Self {
            embedder,
            vector_store,
            chunk_size,
            chunk_overlap,
            min_chunk_size,
        }
    }

    pub async fn ingest(&self, file_path: &Path, file_name: &str) -> Result<IngestResult> {
        let document = load_document_by_extension(file_path, file_name)?;
        let document_id = document.id.clone();
        let doc_file_name = document.file_name.clone();

        let chunks = split_text_to_chunks(
            &document,
            file_name,
            self.chunk_size,
            self.chunk_overlap,
            self.min_chunk_size,
        );
        let total_chunks = chunks.len();

        if chunks.is_empty() {
            return Ok(IngestResult {
                document_id,
                file_name: doc_file_name,
                chunk_count: 0,
            });
        }

        // Process in batches to limit memory for large documents
        for batch_chunks in chunks.chunks(EMBED_BATCH_SIZE) {
            let texts: Vec<String> = batch_chunks.iter().map(|c| c.content.clone()).collect();
            let vectors = self.embedder.embed_texts(&texts).await?;
            self.vector_store.upsert_chunks(batch_chunks, &vectors).await?;
            tracing::debug!(
                "Upserted batch of {} chunks for '{}'",
                batch_chunks.len(),
                doc_file_name
            );
        }

        Ok(IngestResult {
            document_id,
            file_name: doc_file_name,
            chunk_count: total_chunks,
        })
    }
}
