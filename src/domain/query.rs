use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedChunk {
    pub chunk_id: String,
    pub document_id: String,
    pub file_name: String,
    pub chunk_index: usize,
    pub content: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAnswer {
    pub answer: String,
    pub sources: Vec<RetrievedChunk>,
}
