use rust_rag::domain::Document;
use rust_rag::utils::text_splitter::split_text_to_chunks;

#[test]
fn test_split_long_text_into_multiple_chunks() {
    let doc = Document {
        id: "test-1".to_string(),
        file_name: "test.txt".to_string(),
        content: "A".repeat(1000),
    };
    let chunks = split_text_to_chunks(&doc, 200, 50);
    assert!(chunks.len() > 1, "Long text should produce multiple chunks");
}

#[test]
fn test_chunk_index_starts_at_zero_and_increments() {
    let doc = Document {
        id: "test-2".to_string(),
        file_name: "test.txt".to_string(),
        content: "B".repeat(500),
    };
    let chunks = split_text_to_chunks(&doc, 200, 50);
    for (i, chunk) in chunks.iter().enumerate() {
        assert_eq!(chunk.chunk_index, i, "chunk_index should be sequential");
    }
}

#[test]
fn test_chunk_overlap_less_than_chunk_size() {
    let doc = Document {
        id: "test-3".to_string(),
        file_name: "test.txt".to_string(),
        content: "C".repeat(500),
    };
    // chunk_size=200, chunk_overlap=50 — overlap < size, should not panic
    let chunks = split_text_to_chunks(&doc, 200, 50);
    assert!(!chunks.is_empty());
}

#[test]
fn test_empty_text_returns_empty_vec() {
    let doc = Document {
        id: "test-4".to_string(),
        file_name: "empty.txt".to_string(),
        content: "".to_string(),
    };
    let chunks = split_text_to_chunks(&doc, 200, 50);
    assert!(chunks.is_empty());
}

#[test]
fn test_text_shorter_than_chunk_size_returns_single_chunk() {
    let doc = Document {
        id: "test-5".to_string(),
        file_name: "short.txt".to_string(),
        content: "Hello, world!".to_string(),
    };
    let chunks = split_text_to_chunks(&doc, 500, 80);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].content, "Hello, world!");
    assert_eq!(chunks[0].chunk_index, 0);
}

#[test]
#[should_panic(expected = "chunk_overlap")]
fn test_overlap_equal_or_greater_than_size_panics() {
    let doc = Document {
        id: "test-6".to_string(),
        file_name: "test.txt".to_string(),
        content: "D".repeat(500),
    };
    // overlap >= chunk_size should panic
    split_text_to_chunks(&doc, 100, 100);
}
