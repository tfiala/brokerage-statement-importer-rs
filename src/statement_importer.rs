use crate::path_match::PathMatch;
use anyhow::Result;
use mongodb::{ClientSession, Database, bson::oid::ObjectId};
use std::{path::Path, sync::Arc};
use tokio::sync::Mutex;

use async_trait::async_trait;

#[async_trait]
pub trait StatementImporter {
    /// Returns the name of the importer. It must be unique among all importers.
    fn importer_name(&self) -> &'static str;

    /// Returns whether the given file path might be a match for this importer, without opening the file.
    ///
    /// If the filename matches a pattern that could be content that the importer can handle, returns
    /// `PathMatch::Match`. `content_matches` will be called to confirm the match with the actual
    /// file contents.
    async fn path_may_match(&self, path: &Path) -> PathMatch;

    /// Returns whether the given content string matches the importer.
    ///
    /// This is called after `path_may_match` returns `PathMatch::Match`.  If this returns
    /// `PathMatch::Match`, the importer's `import` method will be called to import the content.'
    async fn content_matches(&self, content: &str) -> PathMatch;

    /// Imports the content string into the brokerage database.
    ///
    /// If `session` is `None`, the entirety of the import should be done in a single transaction
    /// that covers just this import call.
    /// If `session` is `Some`, the import should be performed in the context of the provided session.
    async fn import(
        &self,
        content: &str,
        db: &Database,
        session: Option<Arc<Mutex<ClientSession>>>,
        source_id: ObjectId,
    ) -> Result<()>;
}
