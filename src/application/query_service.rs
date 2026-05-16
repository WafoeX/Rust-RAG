use anyhow::Result;
use std::sync::Arc;

use crate::application::prompt_builder::PromptBuilder;
use crate::domain::{
    ports::{ChatMessage, Embedder, LlmClient, VectorStore},
    QueryAnswer,
};
use crate::utils::re_rank::mmr_rerank;

pub struct QueryService {
    embedder: Arc<dyn Embedder>,
    vector_store: Arc<dyn VectorStore>,
    llm_client: Arc<dyn LlmClient>,
    prompt_builder: PromptBuilder,
    default_top_k: usize,
    mmr_lambda: f32,
    min_score: f32,
}

impl QueryService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        embedder: Arc<dyn Embedder>,
        vector_store: Arc<dyn VectorStore>,
        llm_client: Arc<dyn LlmClient>,
        prompt_builder: PromptBuilder,
        default_top_k: usize,
        mmr_lambda: f32,
        min_score: f32,
    ) -> Self {
        Self {
            embedder,
            vector_store,
            llm_client,
            prompt_builder,
            default_top_k,
            mmr_lambda,
            min_score,
        }
    }

    pub async fn query(
        &self,
        question: &str,
        top_k: Option<usize>,
        history: &[ChatMessage],
    ) -> Result<QueryAnswer> {
        let top_k = top_k.unwrap_or(self.default_top_k);

        // Fetch 2x candidates for MMR to select from
        let fetch_k = (top_k * 2).min(100);

        let embedder = self.embedder.clone();
        let vector_store = self.vector_store.clone();
        let question_owned = question.to_string();

        let (_query_vector, candidates) = tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let qv = embedder.embed_query(&question_owned).await?;
                let chunks = vector_store.search(&qv, fetch_k, None).await?;
                Ok::<_, anyhow::Error>((qv, chunks))
            })
        })
        .await??;

        // Filter by minimum score threshold
        let filtered: Vec<_> = candidates
            .into_iter()
            .filter(|c| c.score >= self.min_score)
            .collect();

        tracing::info!(
            "Query returned {} chunks ({} after threshold), re-ranking to {} for question: {}",
            fetch_k,
            filtered.len(),
            top_k,
            question
        );

        // MMR re-rank for diversity
        let chunks = mmr_rerank(filtered, top_k, self.mmr_lambda);

        let prompt = self.prompt_builder.build(question, &chunks, history);
        let answer = self
            .llm_client
            .generate_answer_with_history(&prompt.system, history, &prompt.user)
            .await?;

        Ok(QueryAnswer {
            answer,
            sources: chunks,
        })
    }
}
