use anyhow::Result;
use std::path::PathBuf;

pub fn filter_unimported_files(paths: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    Ok(paths.clone())
}
