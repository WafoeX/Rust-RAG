use anyhow::{Context, Result};
use std::path::Path;

use crate::domain::{ports::DocumentLoader, Document};

pub struct TxtLoader;

impl DocumentLoader for TxtLoader {
    fn load(&self, file_path: &Path, file_name: &str) -> Result<Document> {
        let content = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read TXT file: {:?}", file_path))?;

        Ok(Document {
            id: uuid::Uuid::new_v4().to_string(),
            file_name: file_name.to_string(),
            content,
        })
    }
}
