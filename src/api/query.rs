use axum::{extract::State, Json};
use std::sync::Arc;

use crate::api::dto::{QueryRequest, QueryResponse, SourceDto};
use crate::error::AppError;
use crate::state::AppState;

pub async fn query(
    State(state): State<Arc<AppState>>,
    Json(request): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, AppError> {
    if request.question.trim().is_empty() {
        return Err(AppError::bad_request("question must not be empty"));
    }

    let result = state
        .query_service
        .query(&request.question, request.top_k)
        .await?;

    let sources = result
        .sources
        .into_iter()
        .map(|c| SourceDto {
            file_name: c.file_name,
            chunk_index: c.chunk_index,
            content: c.content,
            score: c.score,
        })
        .collect();

    Ok(Json(QueryResponse {
        answer: result.answer,
        sources,
    }))
}
