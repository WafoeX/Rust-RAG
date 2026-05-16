use anyhow::Result;
use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::api::{health, query, upload};
use crate::application::{IngestService, PromptBuilder, QueryService};
use crate::config::AppConfig;
use crate::domain::ports::VectorStore;
use crate::infrastructure::embedding::FastEmbedder;
use crate::infrastructure::llm::DeepSeekClient;
use crate::infrastructure::vector_store::QdrantVectorStore;
use crate::state::AppState;

pub async fn build_app(config: AppConfig) -> Result<Router> {
    // Set HF endpoint for model download before initializing FastEmbedder.
    // This is read by hf-hub internally; must be set at the process level.
    std::env::set_var("HF_ENDPOINT", &config.hf_endpoint);
    std::env::set_var("HF_HOME", &config.hf_cache_dir);
    // Limit ONNX Runtime thread count to reduce memory (default uses all cores).
    // Each thread allocates its own memory arena for computation.
    std::env::set_var("RAYON_NUM_THREADS", "2");
    std::env::set_var("OMP_NUM_THREADS", "2");
    tracing::info!("HF_ENDPOINT set to: {}", config.hf_endpoint);

    // Create embedder
    tracing::info!("Initializing FastEmbedder (first run will download the model)...");
    let embedder = FastEmbedder::try_new().await?;
    let embedding_dim = embedder.embedding_dim();
    tracing::info!("Embedding dimension: {}", embedding_dim);
    let embedder = Arc::new(embedder);

    // Create vector store
    let vector_store = QdrantVectorStore::new(&config).await?;
    vector_store
        .create_collection_if_not_exists(embedding_dim)
        .await?;
    let vector_store = Arc::new(vector_store);

    // Create LLM client
    let llm_client = DeepSeekClient::new(&config);
    let llm_client = Arc::new(llm_client);

    // Load system prompt
    let system_prompt =
        std::fs::read_to_string("prompts/rag_system_prompt.md").unwrap_or_else(|_| {
            tracing::warn!("prompts/rag_system_prompt.md not found, using default prompt");
            "你是一个严谨的知识库问答助手。请只根据给定资料回答用户问题。".to_string()
        });
    let prompt_builder = PromptBuilder::new(system_prompt);

    // Create services
    let ingest_service = Arc::new(IngestService::new(
        embedder.clone(),
        vector_store.clone(),
        config.chunk_size,
        config.chunk_overlap,
        config.min_chunk_size,
    ));

    let query_service = Arc::new(QueryService::new(
        embedder,
        vector_store,
        llm_client,
        prompt_builder,
        config.rag_top_k,
        config.mmr_lambda,
        config.min_score,
    ));

    let state = Arc::new(AppState {
        ingest_service,
        query_service,
    });

    let router = Router::new()
        .route("/health", get(health::health))
        .route("/api/documents/upload", post(upload::upload_document))
        .route("/api/query", post(query::query))
        .route_service("/", ServeFile::new("static/index.html"))
        .nest_service("/static", ServeDir::new("static"))
        .layer(ConcurrencyLimitLayer::new(8))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // 100MB for file uploads
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    Ok(router)
}
