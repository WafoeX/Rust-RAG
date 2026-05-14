use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::config::AppConfig;
use crate::domain::ports::LlmClient;

pub struct DeepSeekClient {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl DeepSeekClient {
    pub fn new(config: &AppConfig) -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(2)
            .pool_idle_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to build reqwest client");

        Self {
            client,
            api_key: config.deepseek_api_key.clone(),
            base_url: config.deepseek_base_url.clone(),
            model: config.deepseek_model.clone(),
        }
    }
}

#[async_trait]
impl LlmClient for DeepSeekClient {
    async fn generate_answer(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url);

        let body = json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "stream": false,
            "temperature": 0.2
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .context("Failed to send request to DeepSeek API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("DeepSeek API error ({}): {}", status, body);
        }

        let json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse DeepSeek API response")?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .context("Unexpected DeepSeek API response structure")?
            .to_string();

        Ok(content)
    }
}
