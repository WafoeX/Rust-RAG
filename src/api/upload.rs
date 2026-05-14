use axum::{
    extract::{Multipart, State},
    Json,
};
use std::sync::Arc;

use crate::api::dto::UploadResponse;
use crate::error::AppError;
use crate::state::AppState;

pub async fn upload_document(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, AppError> {
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::warn!("Multipart next_field error: {}", e);
        AppError::bad_request(format!("Failed to parse multipart form: {}", e))
    })? {
        let field_name = field.name().unwrap_or("").to_string();
        if field_name == "file" {
            let file_name = field.file_name().unwrap_or("unknown").to_string();

            let data = field.bytes().await.map_err(|e| {
                tracing::warn!("Multipart bytes error for '{}': {}", file_name, e);
                AppError::bad_request(format!("Failed to read file data: {}", e))
            })?;

            // Save to data/raw/
            let raw_dir = std::path::Path::new("data/raw");
            std::fs::create_dir_all(raw_dir).map_err(|e| AppError::internal(e.to_string()))?;

            let file_path = raw_dir.join(&file_name);
            std::fs::write(&file_path, &data).map_err(|e| AppError::internal(e.to_string()))?;

            tracing::info!("Saved file to {}, size: {} bytes", file_path.display(), data.len());

            // Call ingest service
            let result = state.ingest_service.ingest(&file_path, &file_name).await?;

            tracing::info!(
                "Ingested document '{}' with {} chunks",
                result.file_name,
                result.chunk_count
            );

            return Ok(Json(UploadResponse {
                document_id: result.document_id,
                file_name: result.file_name,
                chunk_count: result.chunk_count,
            }));
        }
    }

    Err(AppError::bad_request(
        "No 'file' field found in multipart form data. Use field name 'file' in your form.",
    ))
}
