pub mod ibkr_flex_statement_importer;
pub mod importer_registry;
pub mod path_match;
pub mod statement_importer;
mod writers;

use anyhow::Result;
use std::path::PathBuf;

pub fn filter_unimported_files(paths: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    Ok(paths.clone())
}
