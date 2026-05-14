use anyhow::{Context, Result};
use std::path::Path;

use crate::domain::{ports::DocumentLoader, Document};

pub struct MarkdownLoader;

impl DocumentLoader for MarkdownLoader {
    fn load(&self, file_path: &Path, file_name: &str) -> Result<Document> {
        let raw = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read Markdown file: {:?}", file_path))?;

        // Use pulldown-cmark to extract plain text
        let parser = pulldown_cmark::Parser::new(&raw);
        let mut content = String::new();
        pulldown_cmark::html::push_html(&mut content, parser);
        // The HTML still has tags; strip them simply
        let content = strip_html_tags(&content);

        Ok(Document {
            id: uuid::Uuid::new_v4().to_string(),
            file_name: file_name.to_string(),
            content: content.trim().to_string(),
        })
    }
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
            if result.is_empty() || !result.ends_with(' ') {
                result.push(' ');
            }
        } else if !in_tag {
            result.push(ch);
        }
    }

    // Decode common HTML entities
    let result = result.replace("&amp;", "&");
    let result = result.replace("&lt;", "<");
    let result = result.replace("&gt;", ">");
    let result = result.replace("&quot;", "\"");
    let result = result.replace("&#39;", "'");

    // Collapse multiple spaces and blank lines
    let mut cleaned = String::new();
    let mut prev_was_newline = false;
    let mut blank_count = 0;

    for line in result.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                cleaned.push('\n');
                prev_was_newline = true;
            }
        } else {
            blank_count = 0;
            if prev_was_newline && !cleaned.is_empty() {
                cleaned.push('\n');
            }
            cleaned.push_str(trimmed);
            cleaned.push('\n');
            prev_was_newline = false;
        }
    }

    cleaned.trim().to_string()
}
