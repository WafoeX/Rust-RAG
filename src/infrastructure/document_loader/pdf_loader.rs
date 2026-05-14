use anyhow::{Context, Result};
use std::path::Path;

use crate::domain::{ports::DocumentLoader, Document};

pub struct PdfLoader;

impl DocumentLoader for PdfLoader {
    fn load(&self, file_path: &Path, file_name: &str) -> Result<Document> {
        let content = pdf_extract::extract_text(file_path)
            .with_context(|| format!("Failed to extract text from PDF: {:?}", file_path))?;

        Ok(Document {
            id: uuid::Uuid::new_v4().to_string(),
            file_name: file_name.to_string(),
            content,
        })
    }
}
