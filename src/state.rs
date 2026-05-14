use std::sync::Arc;

use crate::application::{IngestService, QueryService};

pub struct AppState {
    pub ingest_service: Arc<IngestService>,
    pub query_service: Arc<QueryService>,
}
