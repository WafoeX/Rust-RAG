# Rust RAG Knowledge Base

A modular, decoupled Retrieval-Augmented Generation (RAG) system built with Rust.

## Features

- Upload TXT, Markdown, and PDF documents
- Automatic text chunking with configurable size and overlap
- Local embedding generation via FastEmbed (ONNX)
- Vector storage in Qdrant
- Question answering powered by DeepSeek API
- Clean, modular architecture with trait-based abstractions

## Tech Stack

- **Web framework**: Axum + Tokio
- **Embeddings**: FastEmbed (all-MiniLM-L6-v2)
- **Vector store**: Qdrant
- **LLM**: DeepSeek API
- **Document parsing**: pdf-extract, pulldown-cmark

## Prerequisites

- Rust toolchain (1.80+)
- Docker (for Qdrant)

## Setup

### 1. Start Qdrant

```bash
docker run -p 6333:6333 -p 6334:6334 \
  -v "$(pwd)/qdrant_storage:/qdrant/storage:z" \
  qdrant/qdrant
```

### 2. Configure environment

```bash
cp .env.example .env
```

Edit `.env` and set your DeepSeek API key:

```
DEEPSEEK_API_KEY=sk-your-actual-key
```

### 3. Run the server

```bash
cargo run
```

The server starts at `http://127.0.0.1:3000`.

On first run, FastEmbed will download the embedding model (~90MB). This happens automatically.

## API Endpoints

### Health Check

```bash
curl http://127.0.0.1:3000/health
```

### Upload a Document

```bash
curl -X POST http://127.0.0.1:3000/api/documents/upload \
  -F "file=@./examples/sample.md"
```

Supported formats: `.txt`, `.md`, `.markdown`, `.pdf`

### Ask a Question

```bash
curl -X POST http://127.0.0.1:3000/api/query \
  -H "Content-Type: application/json" \
  -d '{"question":"这份文档主要讲了什么？","top_k":5}'
```

## Configuration

All configuration is in `.env`:

| Variable | Default | Description |
|----------|---------|-------------|
| `APP_HOST` | `127.0.0.1` | Server host |
| `APP_PORT` | `3000` | Server port |
| `QDRANT_URL` | `http://localhost:6334` | Qdrant server URL |
| `QDRANT_COLLECTION` | `rust_rag_chunks` | Qdrant collection name |
| `DEEPSEEK_API_KEY` | *(required)* | DeepSeek API key |
| `DEEPSEEK_BASE_URL` | `https://api.deepseek.com` | DeepSeek API base URL |
| `DEEPSEEK_MODEL` | `deepseek-v4-flash` | Model name |
| `RAG_TOP_K` | `5` | Default number of chunks to retrieve |
| `CHUNK_SIZE` | `500` | Characters per chunk |
| `CHUNK_OVERLAP` | `80` | Overlap between chunks |

## Project Structure

```
src/
  main.rs              # Entry point
  app.rs               # Dependency assembly & router
  config.rs            # Configuration from .env
  error.rs             # Unified error handling
  state.rs             # Application state
  api/                 # HTTP handlers & DTOs
  application/         # Service layer (ingest, query, prompt)
  domain/              # Core types & trait abstractions
  infrastructure/      # External integrations (Embedder, Qdrant, LLM, loaders)
  utils/               # Text splitting, file helpers
tests/                 # Integration tests
```

## Running Tests

```bash
cargo test
cargo fmt --check
cargo clippy
```
