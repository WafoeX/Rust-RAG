use std::collections::HashMap;

use anyhow::{Context, Result};
use async_trait::async_trait;
use qdrant_client::qdrant::{
    value::Kind, CreateCollectionBuilder, Distance, PointStruct, QueryPointsBuilder,
    UpsertPointsBuilder, Value, VectorParamsBuilder,
};
use qdrant_client::{Payload, Qdrant};

use crate::config::AppConfig;
use crate::domain::{ports::VectorStore, DocumentChunk, EmbeddingVector, RetrievedChunk};

pub struct QdrantVectorStore {
    client: Qdrant,
    collection_name: String,
}

impl QdrantVectorStore {
    pub async fn new(config: &AppConfig) -> Result<Self> {
        let client = Qdrant::from_url(&config.qdrant_url).build()?;
        Ok(Self {
            client,
            collection_name: config.qdrant_collection.clone(),
        })
    }
}

#[async_trait]
impl VectorStore for QdrantVectorStore {
    async fn create_collection_if_not_exists(&self, vector_size: u64) -> Result<()> {
        let exists = self
            .client
            .collection_exists(&self.collection_name)
            .await
            .context("Failed to check if Qdrant collection exists")?;

        if exists {
            return Ok(());
        }

        self.client
            .create_collection(
                CreateCollectionBuilder::new(&self.collection_name)
                    .vectors_config(VectorParamsBuilder::new(vector_size, Distance::Cosine)),
            )
            .await
            .context("Failed to create Qdrant collection")?;

        tracing::info!(
            "Created Qdrant collection '{}' with vector size {}",
            self.collection_name,
            vector_size
        );

        Ok(())
    }

    async fn upsert_chunks(
        &self,
        chunks: &[DocumentChunk],
        vectors: &[EmbeddingVector],
    ) -> Result<()> {
        if chunks.len() != vectors.len() {
            anyhow::bail!(
                "chunks and vectors length mismatch: {} vs {}",
                chunks.len(),
                vectors.len()
            );
        }

        let mut points = Vec::with_capacity(chunks.len());
        for (i, (chunk, vector)) in chunks.iter().zip(vectors.iter()).enumerate() {
            let payload: Payload = serde_json::json!({
                "chunk_id": chunk.id,
                "document_id": chunk.document_id,
                "file_name": chunk.file_name,
                "chunk_index": chunk.chunk_index,
                "content": chunk.content,
            })
            .try_into()
            .context("Failed to convert payload")?;

            let point = PointStruct::new(i as u64, vector.values.clone(), payload);
            points.push(point);
        }

        self.client
            .upsert_points(UpsertPointsBuilder::new(&self.collection_name, points))
            .await
            .context("Failed to upsert points to Qdrant")?;

        Ok(())
    }

    async fn search(
        &self,
        query_vector: &EmbeddingVector,
        top_k: usize,
    ) -> Result<Vec<RetrievedChunk>> {
        let response = self
            .client
            .query(
                QueryPointsBuilder::new(&self.collection_name)
                    .query(query_vector.values.clone())
                    .limit(top_k as u64)
                    .with_payload(true),
            )
            .await
            .context("Failed to query Qdrant")?;

        let chunks = response
            .result
            .into_iter()
            .map(|scored_point| {
                let payload = &scored_point.payload;
                RetrievedChunk {
                    chunk_id: get_payload_str(payload, "chunk_id"),
                    document_id: get_payload_str(payload, "document_id"),
                    file_name: get_payload_str(payload, "file_name"),
                    chunk_index: get_payload_u64(payload, "chunk_index") as usize,
                    content: get_payload_str(payload, "content"),
                    score: scored_point.score,
                }
            })
            .collect();

        Ok(chunks)
    }
}

fn get_payload_str(payload: &HashMap<String, Value>, key: &str) -> String {
    payload
        .get(key)
        .and_then(|v| match &v.kind {
            Some(Kind::StringValue(s)) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

fn get_payload_u64(payload: &HashMap<String, Value>, key: &str) -> u64 {
    payload
        .get(key)
        .and_then(|v| match &v.kind {
            Some(Kind::IntegerValue(i)) => Some(*i as u64),
            _ => None,
        })
        .unwrap_or(0)
}
