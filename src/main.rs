use anyhow::Result;
use rust_rag::app;
use rust_rag::config;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = config::AppConfig::from_env()?;

    let host = config.app_host.clone();
    let port = config.app_port;

    let router = app::build_app(config).await?;

    let addr = format!("{}:{}", host, port);
    tracing::info!("Server starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
