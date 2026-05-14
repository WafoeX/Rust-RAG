use uuid::Uuid;

use crate::domain::{Document, DocumentChunk};

pub fn split_text_to_chunks(
    document: &Document,
    chunk_size: usize,
    chunk_overlap: usize,
) -> Vec<DocumentChunk> {
    if document.content.is_empty() || chunk_size == 0 {
        return vec![];
    }

    assert!(
        chunk_overlap < chunk_size,
        "chunk_overlap ({chunk_overlap}) must be less than chunk_size ({chunk_size})"
    );

    let chars: Vec<char> = document.content.chars().collect();
    let mut chunks = Vec::new();
    let mut start = 0;

    while start < chars.len() {
        let end = (start + chunk_size).min(chars.len());
        let chunk_content: String = chars[start..end].iter().collect();

        chunks.push(DocumentChunk {
            id: Uuid::new_v4().to_string(),
            document_id: document.id.clone(),
            file_name: document.file_name.clone(),
            chunk_index: chunks.len(),
            content: chunk_content,
        });

        if end >= chars.len() {
            break;
        }

        start = end - chunk_overlap;
    }

    chunks
}
