use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub app_host: String,
    pub app_port: u16,
    pub qdrant_url: String,
    pub qdrant_collection: String,
    pub deepseek_api_key: String,
    pub deepseek_base_url: String,
    pub deepseek_model: String,
    pub rag_top_k: usize,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub min_chunk_size: usize,
    pub mmr_lambda: f32,
    pub min_score: f32,
    pub hf_endpoint: String,
    pub hf_cache_dir: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let deepseek_api_key = std::env::var("DEEPSEEK_API_KEY")
            .context("DEEPSEEK_API_KEY must be set in .env file")?;

        Ok(Self {
            app_host: std::env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            app_port: std::env::var("APP_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .context("APP_PORT must be a valid u16")?,
            qdrant_url: std::env::var("QDRANT_URL")
                .unwrap_or_else(|_| "http://localhost:6334".to_string()),
            qdrant_collection: std::env::var("QDRANT_COLLECTION")
                .unwrap_or_else(|_| "rust_rag_chunks".to_string()),
            deepseek_api_key,
            deepseek_base_url: std::env::var("DEEPSEEK_BASE_URL")
                .unwrap_or_else(|_| "https://api.deepseek.com".to_string()),
            deepseek_model: std::env::var("DEEPSEEK_MODEL")
                .unwrap_or_else(|_| "deepseek-v4-flash".to_string()),
            rag_top_k: std::env::var("RAG_TOP_K")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .context("RAG_TOP_K must be a valid usize")?,
            chunk_size: std::env::var("CHUNK_SIZE")
                .unwrap_or_else(|_| "500".to_string())
                .parse()
                .context("CHUNK_SIZE must be a valid usize")?,
            chunk_overlap: std::env::var("CHUNK_OVERLAP")
                .unwrap_or_else(|_| "80".to_string())
                .parse()
                .context("CHUNK_OVERLAP must be a valid usize")?,
            min_chunk_size: std::env::var("MIN_CHUNK_SIZE")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .context("MIN_CHUNK_SIZE must be a valid usize")?,
            mmr_lambda: std::env::var("MMR_LAMBDA")
                .unwrap_or_else(|_| "0.7".to_string())
                .parse()
                .context("MMR_LAMBDA must be a valid f32")?,
            min_score: std::env::var("MIN_SCORE")
                .unwrap_or_else(|_| "0.0".to_string())
                .parse()
                .context("MIN_SCORE must be a valid f32")?,
            hf_endpoint: std::env::var("HF_ENDPOINT")
                .unwrap_or_else(|_| "https://huggingface.co".to_string()),
            hf_cache_dir: std::env::var("HF_CACHE_DIR")
                .unwrap_or_else(|_| "./model_cache".to_string()),
        })
    }
}
