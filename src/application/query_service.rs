use anyhow::Result;
use std::sync::Arc;

use crate::application::prompt_builder::PromptBuilder;
use crate::domain::{
    ports::{Embedder, LlmClient, VectorStore},
    QueryAnswer,
};

pub struct QueryService {
    embedder: Arc<dyn Embedder>,
    vector_store: Arc<dyn VectorStore>,
    llm_client: Arc<dyn LlmClient>,
    prompt_builder: PromptBuilder,
    default_top_k: usize,
}

impl QueryService {
    pub fn new(
        embedder: Arc<dyn Embedder>,
        vector_store: Arc<dyn VectorStore>,
        llm_client: Arc<dyn LlmClient>,
        prompt_builder: PromptBuilder,
        default_top_k: usize,
    ) -> Self {
        Self {
            embedder,
            vector_store,
            llm_client,
            prompt_builder,
            default_top_k,
        }
    }

    pub async fn query(&self, question: &str, top_k: Option<usize>) -> Result<QueryAnswer> {
        let top_k = top_k.unwrap_or(self.default_top_k);

        let query_vector = self.embedder.embed_query(question).await?;
        let chunks = self.vector_store.search(&query_vector, top_k).await?;

        tracing::info!(
            "Query returned {} chunks for question: {}",
            chunks.len(),
            question
        );

        let prompt = self.prompt_builder.build(question, &chunks);
        let answer = self
            .llm_client
            .generate_answer(&prompt.system, &prompt.user)
            .await?;

        Ok(QueryAnswer {
            answer,
            sources: chunks,
        })
    }
}
