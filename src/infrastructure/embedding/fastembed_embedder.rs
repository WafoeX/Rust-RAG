use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use fastembed::{
    InitOptionsUserDefined, Pooling, QuantizationMode, TextEmbedding, TokenizerFiles,
    UserDefinedEmbeddingModel,
};

use crate::domain::{ports::Embedder, EmbeddingVector};

const MODEL_REPO: &str = "Qdrant/all-MiniLM-L6-v2-onnx";
const MODEL_REVISION: &str = "main";

const REQUIRED_FILES: &[&str] = &[
    "model.onnx",
    "tokenizer.json",
    "config.json",
    "tokenizer_config.json",
    "special_tokens_map.json",
];

pub struct FastEmbedder {
    model: Mutex<TextEmbedding>,
    dimension: usize,
}

impl FastEmbedder {
    pub async fn try_new() -> Result<Self> {
        let endpoint =
            std::env::var("HF_ENDPOINT").unwrap_or_else(|_| "https://huggingface.co".to_string());
        let cache_base = std::env::var("HF_HOME").unwrap_or_else(|_| "./model_cache".to_string());

        let model_dir = PathBuf::from(&cache_base).join("all-MiniLM-L6-v2");
        std::fs::create_dir_all(&model_dir).context("Failed to create model cache directory")?;

        // Download any missing model files
        download_model_files(&endpoint, &model_dir).await?;

        // Read model files from cache
        let onnx_path = model_dir.join("model.onnx");
        let onnx_file = std::fs::read(&onnx_path)
            .with_context(|| format!("Failed to read {}", onnx_path.display()))?;

        let tokenizer_files = TokenizerFiles {
            tokenizer_file: read_model_file(&model_dir, "tokenizer.json")?,
            config_file: read_model_file(&model_dir, "config.json")?,
            special_tokens_map_file: read_model_file(&model_dir, "special_tokens_map.json")?,
            tokenizer_config_file: read_model_file(&model_dir, "tokenizer_config.json")?,
        };

        let user_model = UserDefinedEmbeddingModel {
            onnx_file,
            external_initializers: Vec::new(),
            tokenizer_files,
            pooling: Some(Pooling::Mean),
            quantization: QuantizationMode::None,
            output_key: None,
        };

        let options = InitOptionsUserDefined::new();
        let mut model = TextEmbedding::try_new_from_user_defined(user_model, options)?;

        let dim = model
            .embed(["dimension check"], None)
            .map(|v| v.first().map(|e| e.len()).unwrap_or(384))
            .unwrap_or(384);

        Ok(Self {
            model: Mutex::new(model),
            dimension: dim,
        })
    }

    pub fn embedding_dim(&self) -> u64 {
        self.dimension as u64
    }
}

fn read_model_file(model_dir: &std::path::Path, filename: &str) -> Result<Vec<u8>> {
    let path = model_dir.join(filename);
    std::fs::read(&path).with_context(|| format!("Failed to read {}", path.display()))
}

async fn download_model_files(endpoint: &str, model_dir: &std::path::Path) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .context("Failed to create HTTP client")?;

    for filename in REQUIRED_FILES {
        let dest = model_dir.join(filename);
        if dest.exists() {
            tracing::info!("Model file already cached: {}", filename);
            continue;
        }

        let url = format!(
            "{}/{}/resolve/{}/{}",
            endpoint.trim_end_matches('/'),
            MODEL_REPO,
            MODEL_REVISION,
            filename
        );

        tracing::info!("Downloading model file: {}", filename);
        let response = client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("Failed to download model file: {url}"))?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to download {} (HTTP {}): {url}",
                filename,
                response.status()
            );
        }

        let bytes = response
            .bytes()
            .await
            .with_context(|| format!("Failed to read response body for: {url}"))?;

        std::fs::write(&dest, &bytes)
            .with_context(|| format!("Failed to write model file: {}", dest.display()))?;

        tracing::info!(
            "Downloaded model file: {} ({} bytes)",
            filename,
            bytes.len()
        );
    }

    Ok(())
}

#[async_trait]
impl Embedder for FastEmbedder {
    async fn embed_texts(&self, texts: &[String]) -> Result<Vec<EmbeddingVector>> {
        let texts_ref: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
        let embeddings = self.model.lock().unwrap().embed(&texts_ref, None)?;
        Ok(embeddings
            .into_iter()
            .map(|v| EmbeddingVector { values: v })
            .collect())
    }

    async fn embed_query(&self, query: &str) -> Result<EmbeddingVector> {
        let embeddings = self.model.lock().unwrap().embed([query], None)?;
        let vec = embeddings.into_iter().next().unwrap_or_default();
        Ok(EmbeddingVector { values: vec })
    }
}
