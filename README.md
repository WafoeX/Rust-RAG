# Rust RAG

A modular, high-performance Retrieval-Augmented Generation (RAG) system built in Rust. Upload documents, ask questions, and get answers grounded in your knowledge base — with natural boundary chunking, MMR-based re-ranking, and multi-turn conversation support.

## Features

- **Document Ingestion** — Upload TXT, Markdown, and PDF files via drag-and-drop or API
- **Natural Boundary Chunking** — Recursive text splitter that respects paragraphs, sentences, and clause boundaries before falling back to character-level splits. Markdown-aware mode preserves heading structure.
- **Local Embeddings** — ONNX-based embedding via FastEmbed (`all-MiniLM-L6-v2`, 384-dim), no external API calls
- **Vector Search** — Cosine similarity search in Qdrant with configurable top-K
- **MMR Re-Ranking** — Maximum Marginal Relevance post-processing to reduce redundancy and ensure diverse retrieval results
- **Score Threshold Filtering** — Discard low-relevance chunks before prompt construction
- **Multi-Turn Conversation** — Conversation history support for follow-up questions with contextual awareness
- **Score-Aware Prompts** — Relevance percentages shown to the LLM so it can weigh evidence quality
- **Citation Enforcement** — System prompt requires explicit source references with fragment numbers
- **Document-Filtered Search** — Optional filtering to search within a specific document
- **Clean Architecture** — Trait-based abstractions (`Embedder`, `VectorStore`, `LlmClient`) for easy swapping of components

## Architecture

```
┌──────────┐    ┌──────────────┐    ┌─────────┐    ┌──────────┐
│  Axum    │───▶│ QueryService │───▶│ Qdrant  │───▶│ DeepSeek │
│  Router  │    │ + MMR rerank │    │ Vector  │    │   LLM    │
└──────────┘    └──────────────┘    │ Store   │    └──────────┘
      │               │             └─────────┘          │
      ▼               ▼                                  │
┌──────────┐    ┌──────────────┐                         │
│ Ingest   │───▶│ FastEmbedder │                         │
│ Service  │    │  (ONNX 384d) │                         │
└──────────┘    └──────────────┘                         │
      │                                                  │
      ▼                                                  ▼
┌──────────┐                                   ┌────────────────┐
│ Recursive│                                   │ PromptBuilder  │
│ Text     │                                   │ + history      │
│ Splitter │                                   │ + scores       │
└──────────┘                                   └────────────────┘
```

### Data Flow

**Ingestion**: Upload → Document Loader (txt/md/pdf) → Recursive Text Splitter (paragraphs → sentences → clauses → chars) → FastEmbedder (384-dim vectors) → Qdrant (cosine similarity)

**Query**: Question → FastEmbedder (query vector) → Qdrant Search (top_k × 2) → Score Threshold Filter → MMR Re-Rank (top_k) → Prompt Builder (+ history, + scores) → DeepSeek → Answer + Sources

## Quick Start

### Prerequisites

- Rust 1.80+
- Docker (for Qdrant)

### 1. Start Qdrant

```bash
docker run -p 6333:6333 -p 6334:6334 \
  -v "$(pwd)/qdrant_storage:/qdrant/storage:z" \
  qdrant/qdrant
```

### 2. Configure Environment

```bash
cp .env.example .env
```

Edit `.env` and set your DeepSeek API key:

```env
DEEPSEEK_API_KEY=sk-your-actual-key
```

### 3. Run

```bash
cargo run
```

Open `http://127.0.0.1:3000`. On first run, the embedding model (~90 MB) downloads automatically.

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `APP_HOST` | `127.0.0.1` | Server bind address |
| `APP_PORT` | `3000` | Server port |
| `QDRANT_URL` | `http://localhost:6334` | Qdrant server URL (gRPC) |
| `QDRANT_COLLECTION` | `rust_rag_chunks` | Qdrant collection name |
| `DEEPSEEK_API_KEY` | *(required)* | DeepSeek API key |
| `DEEPSEEK_BASE_URL` | `https://api.deepseek.com` | LLM API base URL (OpenAI-compatible) |
| `DEEPSEEK_MODEL` | `deepseek-v4-flash` | LLM model name |
| `RAG_TOP_K` | `5` | Chunks returned after re-ranking |
| `CHUNK_SIZE` | `500` | Max characters per chunk |
| `CHUNK_OVERLAP` | `80` | Character overlap between adjacent chunks |
| `MIN_CHUNK_SIZE` | `100` | Merge chunks smaller than this |
| `MMR_LAMBDA` | `0.7` | MMR relevance-vs-diversity weight (1.0 = pure relevance) |
| `MIN_SCORE` | `0.0` | Minimum cosine similarity threshold (chunks below are discarded) |
| `HF_ENDPOINT` | `https://huggingface.co` | Model download mirror |
| `HF_CACHE_DIR` | `./model_cache` | Model file cache directory |

## API Reference

### `GET /health`

Health check.

```bash
curl http://127.0.0.1:3000/health
# → {"status":"ok"}
```

### `POST /api/documents/upload`

Upload a document. Multipart form with field `file`.

```bash
curl -X POST http://127.0.0.1:3000/api/documents/upload \
  -F "file=@./examples/sample.md"
```

**Response**:
```json
{
  "document_id": "a1b2c3d4-...",
  "file_name": "sample.md",
  "chunk_count": 15
}
```

Supported formats: `.txt`, `.md`, `.markdown`, `.pdf` — up to 100 MB.

### `POST /api/query`

Ask a question with optional conversation history.

```bash
curl -X POST http://127.0.0.1:3000/api/query \
  -H "Content-Type: application/json" \
  -d '{
    "question": "What does the document say about X?",
    "top_k": 5,
    "history": [
      {"role": "user", "content": "Tell me about topic Y"},
      {"role": "assistant", "content": "The document mentions..."}
    ]
  }'
```

**Request fields**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `question` | string | yes | The question to answer |
| `top_k` | integer | no | Chunks to retrieve (default: config value) |
| `history` | array | no | Previous conversation turns |

**Response**:
```json
{
  "answer": "Based on the provided materials...",
  "sources": [
    {
      "file_name": "sample.md",
      "chunk_index": 3,
      "content": "...",
      "score": 0.852
    }
  ]
}
```

## Chunking Strategy

The `RecursiveTextSplitter` uses a priority-ordered separator hierarchy:

| Priority | Separator | Boundary Type |
|----------|-----------|---------------|
| 1 | `\n\n` | Paragraph |
| 2 | `\n#`, `\n##`, ... (markdown) | Heading |
| 3 | `\n` | Line / soft break |
| 4 | `。`, `. `, `！`, `？` | Sentence |
| 5 | `; ` | Clause |
| 6 | ` ` | Word |
| 7 | `""` | Character (fallback) |

After splitting, overlap is applied and orphan chunks below `MIN_CHUNK_SIZE` are merged (respecting `CHUNK_SIZE`).

## MMR Re-Ranking

After vector search returns `top_k × 2` candidates, Maximum Marginal Relevance selects the most diverse `top_k` chunks:

```
MMR = λ × relevance_score - (1 - λ) × max_similarity_to_selected
```

- **λ = 1.0**: Pure relevance (original ranking)
- **λ = 0.7** (default): Mild diversity bias — good for factual QA
- **λ = 0.5**: Balanced — useful for broad exploratory queries

The diversity penalty uses score proximity as a lightweight proxy for content similarity, avoiding the need for cross-encoder models or storing raw embeddings.

## Project Structure

```
src/
  main.rs              # Entry point (tokio multi-thread, 4 workers)
  app.rs               # Dependency assembly & Axum router
  config.rs            # Configuration from environment
  error.rs             # Unified AppError type
  state.rs             # AppState (Arc<IngestService> + Arc<QueryService>)
  lib.rs               # Module re-exports
  api/
    mod.rs
    dto.rs             # Request/response types + ChatMessage
    health.rs          # GET /health
    upload.rs          # POST /api/documents/upload (multipart)
    query.rs           # POST /api/query
  application/
    mod.rs
    ingest_service.rs  # Document ingestion orchestration
    query_service.rs   # Query pipeline + MMR re-ranking
    prompt_builder.rs  # LLM prompt construction with history + scores
  domain/
    mod.rs
    document.rs        # Document, DocumentChunk structs
    embedding.rs       # EmbeddingVector type
    ports.rs           # Embedder, VectorStore, LlmClient traits
    query.rs           # RetrievedChunk, QueryAnswer structs
  infrastructure/
    mod.rs
    document_loader/   # TXT, Markdown, PDF loaders
    embedding/         # FastEmbedder (all-MiniLM-L6-v2 via ONNX)
    llm/               # DeepSeekClient (OpenAI-compatible API)
    vector_store/      # QdrantVectorStore
  utils/
    mod.rs
    text_splitter.rs   # RecursiveTextSplitter (natural boundaries)
    re_rank.rs         # MMR re-ranking
    file.rs            # File system helpers
prompts/
  rag_system_prompt.md # LLM system prompt (Chinese)
static/
  index.html           # Web UI
  app.js               # Frontend logic (upload + chat + history)
  style.css            # Styles
tests/
  prompt_builder_test.rs
  text_splitter_test.rs
```

## Running Tests

```bash
cargo test
cargo fmt --check
cargo clippy
```

## Design Decisions

- **Stateless retrieval, stateful conversation** — Each query triggers a fresh vector search; only Q&A text (not old chunks) is carried forward in conversation history.
- **Trait-based ports** — All external dependencies (embedding, vector store, LLM) are behind traits, making them testable and swappable.
- **No cross-encoder** — MMR uses a lightweight score-proximity proxy instead of a heavy cross-encoder model, keeping latency low.
- **ONNX thread limiting** — `RAYON_NUM_THREADS=2` and `OMP_NUM_THREADS=2` bound ONNX Runtime memory usage.
- **`spawn_blocking` for embedding** — CPU-bound ONNX inference runs off the async runtime to avoid blocking request handling.
- **Single-pass embedding** — All chunk texts are passed to `embed_texts()` in one call, letting ONNX optimize internal batching.
