pub mod markdown_loader;
pub mod pdf_loader;
pub mod txt_loader;

use anyhow::{anyhow, Result};
use std::path::Path;

use crate::domain::{ports::DocumentLoader, Document};

use self::markdown_loader::MarkdownLoader;
use self::pdf_loader::PdfLoader;
use self::txt_loader::TxtLoader;

pub fn load_document_by_extension(file_path: &Path, file_name: &str) -> Result<Document> {
    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "txt" => TxtLoader.load(file_path, file_name),
        "md" | "markdown" => MarkdownLoader.load(file_path, file_name),
        "pdf" => PdfLoader.load(file_path, file_name),
        _ => Err(anyhow!("Unsupported file extension: .{}", extension)),
    }
}
