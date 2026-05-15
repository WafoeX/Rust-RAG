use rust_rag::application::prompt_builder::PromptBuilder;
use rust_rag::domain::RetrievedChunk;

fn make_chunk(id: &str, file_name: &str, chunk_index: usize, content: &str) -> RetrievedChunk {
    RetrievedChunk {
        chunk_id: id.to_string(),
        document_id: format!("doc-{}", id),
        file_name: file_name.to_string(),
        chunk_index,
        content: content.to_string(),
        score: 0.85,
    }
}

#[test]
fn test_prompt_contains_user_question() {
    let builder = PromptBuilder::new("You are a helpful assistant.".to_string());
    let chunks = vec![make_chunk("1", "test.txt", 0, "Some content")];
    let prompt = builder.build("这份文档讲了什么？", &chunks, &[]);
    assert!(
        prompt.user.contains("这份文档讲了什么？"),
        "Prompt should contain the user question"
    );
}

#[test]
fn test_prompt_contains_chunk_content() {
    let builder = PromptBuilder::new("System prompt".to_string());
    let chunks = vec![make_chunk(
        "1",
        "notes.md",
        0,
        "Rust is a systems programming language.",
    )];
    let prompt = builder.build("What is Rust?", &chunks, &[]);
    assert!(
        prompt
            .user
            .contains("Rust is a systems programming language"),
        "Prompt should contain chunk content"
    );
}

#[test]
fn test_prompt_contains_file_name_and_chunk_index() {
    let builder = PromptBuilder::new("System prompt".to_string());
    let chunks = vec![make_chunk("1", "report.pdf", 3, "Financial data")];
    let prompt = builder.build("question?", &chunks, &[]);
    assert!(
        prompt.user.contains("report.pdf"),
        "Should contain file name"
    );
    assert!(
        prompt.user.contains("片段序号：3"),
        "Should contain chunk index"
    );
}

#[test]
fn test_prompt_contains_fragment_label() {
    let builder = PromptBuilder::new("System prompt".to_string());
    let chunks = vec![
        make_chunk("1", "a.txt", 0, "Content A"),
        make_chunk("2", "b.txt", 0, "Content B"),
    ];
    let prompt = builder.build("question?", &chunks, &[]);
    assert!(prompt.user.contains("【资料片段 1】"));
    assert!(prompt.user.contains("【资料片段 2】"));
}

#[test]
fn test_prompt_system_is_preserved() {
    let system = "请只根据给定资料回答，资料不足时说明资料不足。".to_string();
    let builder = PromptBuilder::new(system.clone());
    let prompt = builder.build("question?", &[], &[]);
    assert_eq!(prompt.system, system);
}

#[test]
fn test_prompt_with_empty_chunks_does_not_panic() {
    let builder = PromptBuilder::new("System".to_string());
    let prompt = builder.build("question?", &[], &[]);
    assert!(prompt.user.contains("question?"));
    assert!(!prompt.user.contains("【资料片段"));
}
