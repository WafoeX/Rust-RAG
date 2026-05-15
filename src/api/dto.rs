use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub question: String,
    pub top_k: Option<usize>,
    pub history: Option<Vec<ChatMessage>>,
}

#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub answer: String,
    pub sources: Vec<SourceDto>,
}

#[derive(Debug, Serialize)]
pub struct SourceDto {
    pub file_name: String,
    pub chunk_index: usize,
    pub content: String,
    pub score: f32,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub document_id: String,
    pub file_name: String,
    pub chunk_count: usize,
}
