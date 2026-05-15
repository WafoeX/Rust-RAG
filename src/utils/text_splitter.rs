use uuid::Uuid;

use crate::domain::{Document, DocumentChunk};

/// Splits text recursively trying the most natural boundaries first.
/// Falls back to character-level splitting only as a last resort.
struct RecursiveTextSplitter {
    separators: Vec<String>,
    max_chunk_size: usize,
    chunk_overlap: usize,
    min_chunk_size: usize,
}

impl RecursiveTextSplitter {
    fn new(
        separators: Vec<String>,
        max_chunk_size: usize,
        chunk_overlap: usize,
        min_chunk_size: usize,
    ) -> Self {
        Self {
            separators,
            max_chunk_size,
            chunk_overlap,
            min_chunk_size,
        }
    }

    fn split(&self, text: &str) -> Vec<String> {
        // Try paragraph boundaries (default H1 separator) then fall back
        let raw = self._split_recurse(text, 0);

        // Apply overlap
        let overlapped = self._apply_overlap(&raw);

        // Merge orphan chunks that are too small
        self._merge_small_chunks(&overlapped)
    }

    fn _apply_overlap(&self, chunks: &[String]) -> Vec<String> {
        if self.chunk_overlap == 0 || chunks.len() <= 1 {
            return chunks
                .iter()
                .map(|c| c.trim().to_string())
                .filter(|c| !c.is_empty())
                .collect();
        }

        let trimmed: Vec<String> = chunks
            .iter()
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty())
            .collect();

        if trimmed.len() <= 1 {
            return trimmed;
        }

        let mut result = Vec::with_capacity(trimmed.len());
        for (i, chunk) in trimmed.iter().enumerate() {
            if i == 0 {
                result.push(chunk.clone());
            } else {
                let prev = &trimmed[i - 1];
                let prev_chars: Vec<char> = prev.chars().collect();
                let actual_overlap = self.chunk_overlap.min(prev_chars.len());
                let overlap_start = prev_chars.len() - actual_overlap;
                let overlap_text: String = prev_chars[overlap_start..].iter().collect();

                let mut merged = overlap_text;
                merged.push_str(chunk);
                result.push(merged);
            }
        }
        result
    }

    fn _merge_small_chunks(&self, chunks: &[String]) -> Vec<String> {
        if chunks.is_empty() {
            return vec![];
        }

        let mut result: Vec<String> = Vec::new();
        let mut pending: Option<String> = None;

        for chunk in chunks {
            let chunk = chunk.trim().to_string();
            if chunk.is_empty() {
                continue;
            }

            let char_len = chunk.chars().count();

            if char_len < self.min_chunk_size {
                // Small chunk — try to accumulate into pending
                let would_overflow = if let Some(ref p) = pending {
                    p.chars().count() + 1 + char_len > self.max_chunk_size
                } else {
                    false
                };

                if would_overflow {
                    // Flush pending as its own chunk before starting a new one
                    result.push(pending.take().unwrap());
                    pending = Some(chunk);
                } else if let Some(ref mut p) = pending {
                    p.push('\n');
                    p.push_str(&chunk);
                } else {
                    pending = Some(chunk);
                }
            } else {
                // Normal-sized chunk — merge any pending small chunk before it
                if let Some(p) = pending.take() {
                    let combined_len = p.chars().count() + 1 + char_len;
                    if combined_len <= self.max_chunk_size {
                        let mut merged = p;
                        merged.push('\n');
                        merged.push_str(&chunk);
                        result.push(merged);
                    } else {
                        // Pending and this chunk together exceed max — emit separately
                        result.push(p);
                        result.push(chunk);
                    }
                } else {
                    result.push(chunk);
                }
            }
        }

        // Flush any remaining pending small chunk
        if let Some(p) = pending.take() {
            let can_merge = result
                .last()
                .map(|last| last.chars().count() + 1 + p.chars().count() <= self.max_chunk_size)
                .unwrap_or(false);
            if can_merge {
                let last = result.last_mut().unwrap();
                last.push('\n');
                last.push_str(&p);
            } else {
                result.push(p);
            }
        }

        result
    }

    fn _split_recurse(&self, text: &str, sep_idx: usize) -> Vec<String> {
        // If text already fits, don't split further
        if text.chars().count() <= self.max_chunk_size {
            return if text.trim().is_empty() {
                vec![]
            } else {
                vec![text.to_string()]
            };
        }

        if sep_idx >= self.separators.len() {
            // Last resort: character-level splitting
            return self._char_split(text);
        }

        let separator = &self.separators[sep_idx];

        if separator.is_empty() {
            return self._char_split(text);
        }

        let parts: Vec<&str> = text.split(separator).collect();

        if parts.len() == 1 {
            // This separator didn't match — try the next one
            return self._split_recurse(text, sep_idx + 1);
        }

        let mut result: Vec<String> = Vec::new();
        for part in &parts {
            if part.is_empty() {
                continue;
            }
            let part_len = part.chars().count();
            if part_len <= self.max_chunk_size {
                result.push(part.to_string());
            } else {
                // Too big — try finer separator
                let sub = self._split_recurse(part, sep_idx + 1);
                result.extend(sub);
            }
        }
        result
    }

    fn _char_split(&self, text: &str) -> Vec<String> {
        let chars: Vec<char> = text.chars().collect();
        let mut chunks = Vec::new();
        let mut start = 0;

        while start < chars.len() {
            let end = (start + self.max_chunk_size).min(chars.len());
            let chunk_content: String = chars[start..end].iter().collect();
            chunks.push(chunk_content);
            start = end;
        }

        chunks
    }
}

/// Markdown-aware separators: heading boundaries before paragraph breaks.
fn markdown_separators() -> Vec<String> {
    vec![
        "\n\n".to_string(),
        "\n# ".to_string(),
        "\n## ".to_string(),
        "\n### ".to_string(),
        "\n#### ".to_string(),
        "\n".to_string(),
        "。".to_string(),
        ". ".to_string(),
        "！".to_string(),
        "？".to_string(),
        "; ".to_string(),
        " ".to_string(),
        String::new(), // character-level fallback
    ]
}

/// General-purpose text separators for plain text and PDF.
fn text_separators() -> Vec<String> {
    vec![
        "\n\n".to_string(),
        "\n".to_string(),
        "。".to_string(),
        ". ".to_string(),
        "！".to_string(),
        "？".to_string(),
        "; ".to_string(),
        " ".to_string(),
        String::new(), // character-level fallback
    ]
}

pub fn split_text_to_chunks(
    document: &Document,
    file_name: &str,
    chunk_size: usize,
    chunk_overlap: usize,
    min_chunk_size: usize,
) -> Vec<DocumentChunk> {
    if document.content.is_empty() || chunk_size == 0 {
        return vec![];
    }

    assert!(
        chunk_overlap < chunk_size,
        "chunk_overlap ({chunk_overlap}) must be less than chunk_size ({chunk_size})"
    );

    let extension = file_name
        .rsplit('.')
        .next()
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let separators = if extension == "md" || extension == "markdown" {
        markdown_separators()
    } else {
        text_separators()
    };

    let splitter = RecursiveTextSplitter::new(separators, chunk_size, chunk_overlap, min_chunk_size);
    let chunk_texts = splitter.split(&document.content);

    chunk_texts
        .into_iter()
        .enumerate()
        .map(|(i, content)| DocumentChunk {
            id: Uuid::new_v4().to_string(),
            document_id: document.id.clone(),
            file_name: document.file_name.clone(),
            chunk_index: i,
            content,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Document;

    fn make_doc(content: &str) -> Document {
        Document {
            id: "test-doc".to_string(),
            file_name: "test.txt".to_string(),
            content: content.to_string(),
        }
    }

    #[test]
    fn splits_on_paragraph_boundaries() {
        // Each paragraph is long enough that two won't fit in one chunk
        let p1 = "A".repeat(80);
        let p2 = "B".repeat(80);
        let p3 = "C".repeat(80);
        let content = format!("{}\n\n{}\n\n{}", p1, p2, p3);
        let doc = make_doc(&content);
        // max_chunk_size=100 means two 80-char paragraphs (80+1+80=161) won't fit
        let chunks = split_text_to_chunks(&doc, "test.txt", 100, 0, 100);
        assert!(chunks.len() >= 2, "expected >=2 chunks, got {}", chunks.len());
        // Each chunk should contain at least one of the paragraph markers
        let all_content: String = chunks.iter().map(|c| c.content.as_str()).collect();
        assert!(all_content.contains(&p1));
        assert!(all_content.contains(&p2));
        assert!(all_content.contains(&p3));
    }

    #[test]
    fn splits_on_newlines_when_no_paragraphs() {
        let l1 = "A".repeat(80);
        let l2 = "B".repeat(80);
        let l3 = "C".repeat(80);
        let content = format!("{}\n{}\n{}", l1, l2, l3);
        let doc = make_doc(&content);
        let chunks = split_text_to_chunks(&doc, "test.txt", 100, 0, 100);
        assert!(chunks.len() >= 2, "expected >=2 chunks, got {}", chunks.len());
    }

    #[test]
    fn falls_back_to_chars_for_long_sentence() {
        // Single sentence too long for any separator — must split by char
        let long = "a".repeat(600);
        let doc = make_doc(&format!("短句。{}", long));
        let chunks = split_text_to_chunks(&doc, "test.txt", 500, 0, 100);
        // Should have the short sentence and at least 2 chunks for the long part
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn splits_on_chinese_period() {
        let s1 = format!("这是第一句话。{}", "A".repeat(80));
        let s2 = format!("这是第二句话。{}", "B".repeat(80));
        let s3 = format!("这是第三句话。{}", "C".repeat(80));
        let content = format!("{}{}{}", s1, s2, s3);
        let doc = make_doc(&content);
        // With max_chunk_size=100, each ~90 char sentence ~fits one per chunk
        let chunks = split_text_to_chunks(&doc, "test.txt", 100, 0, 100);
        assert_eq!(chunks.len(), 3);
    }

    #[test]
    fn applies_overlap() {
        let doc = make_doc(
            "这是第一段内容，比较短。\n\n第二段内容稍微长一点，需要更多的文字来填充。",
        );
        let chunks = split_text_to_chunks(&doc, "test.txt", 20, 5, 0);
        // Verify each chunk doesn't exceed max_chunk_size + overlap
        for chunk in &chunks {
            assert!(
                chunk.content.chars().count() <= 30,
                "chunk too long: {}",
                chunk.content.chars().count()
            );
        }
    }

    #[test]
    fn empty_text_returns_empty() {
        let doc = make_doc("");
        let chunks = split_text_to_chunks(&doc, "test.txt", 500, 80, 100);
        assert!(chunks.is_empty());
    }

    #[test]
    fn markdown_preserves_headings() {
        let s1 = format!("## 第一章\n\n{}\n\n", "A".repeat(120));
        let s2 = format!("## 第二章\n\n{}", "B".repeat(120));
        let md = make_doc(&format!("{}{}", s1, s2));
        // max_chunk_size=150 — each section is ~140 chars, should split at ##
        let chunks = split_text_to_chunks(&md, "test.md", 150, 0, 100);
        assert_eq!(chunks.len(), 2, "expected 2 chunks, got {:?}", chunks.iter().map(|c| &c.content).collect::<Vec<_>>());
        assert!(chunks[0].content.contains("第一章"));
        assert!(chunks[1].content.contains("第二章"));
    }

    #[test]
    fn markdown_subsection_split() {
        let overview = format!("## 概述\n\n{}\n\n", "A".repeat(80));
        let detail = format!("### 细节\n\n{}\n\n", "B".repeat(80));
        let more = format!("### 更多\n\n{}", "C".repeat(80));
        let md = make_doc(&format!("{}{}{}", overview, detail, more));
        let chunks = split_text_to_chunks(&md, "test.md", 120, 0, 100);
        // With max_chunk_size=120, each ~90 char section stays separate
        assert!(chunks.len() >= 2);
    }
}
