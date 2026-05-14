use anyhow::{Context, Result};
use std::path::Path;

#[allow(dead_code)]
pub fn ensure_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }
    Ok(())
}
