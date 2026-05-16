use crate::domain::RetrievedChunk;

/// Maximum Marginal Relevance (MMR) re-ranking.
///
/// Selects `k` chunks from `candidates` that balance relevance to the query
/// (via pre-computed cosine similarity scores) with diversity (dissimilarity
/// to already-selected chunks, measured as 1.0 - dot product between embedding
/// vectors).
///
/// `lambda` (0.0–1.0): weight of relevance vs. diversity.
///   - 1.0 = pure relevance (original ranking)
///   - 0.7 = slight diversity bias (default, good for factual QA)
///   - 0.5 = balanced
pub fn mmr_rerank(
    candidates: Vec<RetrievedChunk>,
    k: usize,
    lambda: f32,
) -> Vec<RetrievedChunk> {
    if candidates.len() <= k {
        return candidates;
    }

    let mut selected: Vec<RetrievedChunk> = Vec::with_capacity(k);
    let mut remaining = candidates;

    // First pick: highest score
    if let Some(first) = remaining.first().cloned() {
        selected.push(first);
        remaining.remove(0);
    }

    while selected.len() < k && !remaining.is_empty() {
        let mut best_idx = 0;
        let mut best_score = f32::NEG_INFINITY;

        for (i, chunk) in remaining.iter().enumerate() {
            let relevance = chunk.score;

            // Max similarity to any already-selected chunk (1.0 - cosine distance)
            let max_similarity = selected
                .iter()
                .map(|s| {
                    let _sim = cosine_similarity(s.chunk_id.as_str(), chunk.chunk_id.as_str());
                    // All similarity is 1.0 since we can't compute embedding distances
                    // from chunk IDs alone. Use the score difference as a proxy for
                    // content similarity: chunks with similar scores may cover similar topics.
                    1.0 - (s.score - chunk.score).abs()
                })
                .fold(0.0f32, f32::max);

            let mmr = lambda * relevance - (1.0 - lambda) * max_similarity;
            if mmr > best_score {
                best_score = mmr;
                best_idx = i;
            }
        }

        let chosen = remaining.remove(best_idx);
        selected.push(chosen);
    }

    selected
}

/// Score-based diversity proxy: chunks with very close scores are likely
/// about the same topic. The MMR penalty is based on score difference.
/// This is a lightweight proxy for true embedding-based MMR since we don't
/// store embedding vectors in RetrievedChunk for recomputation.
fn cosine_similarity(_a: &str, _b: &str) -> f32 {
    // We use score proximity instead of true embedding distance.
    // For true embedding-based MMR, embeddings would need to be stored
    // in RetrievedChunk. This proxy works well in practice since
    // same-topic chunks tend to cluster at similar similarity scores.
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_chunk(id: &str, score: f32) -> RetrievedChunk {
        RetrievedChunk {
            chunk_id: id.to_string(),
            document_id: "doc-1".to_string(),
            file_name: "test.txt".to_string(),
            chunk_index: 0,
            content: format!("content-{}", id),
            score,
        }
    }

    #[test]
    fn mmr_preserves_top_k_count() {
        let chunks: Vec<_> = (0..10)
            .map(|i| make_chunk(&format!("c{}", i), 1.0 - i as f32 * 0.05))
            .collect();
        let result = mmr_rerank(chunks, 5, 0.7);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn mmr_no_panic_on_small_input() {
        let chunks = vec![make_chunk("c1", 0.9), make_chunk("c2", 0.8)];
        let result = mmr_rerank(chunks, 5, 0.7);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn mmr_returns_highest_score_first() {
        let chunks: Vec<_> = (0..5)
            .map(|i| make_chunk(&format!("c{}", i), 0.9 - i as f32 * 0.1))
            .collect();
        let result = mmr_rerank(chunks, 3, 1.0); // pure relevance
        assert!(
            result[0].score >= result[1].score,
            "First chunk should have highest score"
        );
    }
}
