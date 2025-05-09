use std::{fs, path::PathBuf};

use crate::path_match::PathMatch;
use crate::statement_importer::StatementImporter;
use anyhow::Result;
use mongodb::{ClientSession, Database, bson::oid::ObjectId};
use tracing::info;

pub struct ImporterRegistry {
    importers: Vec<Box<dyn StatementImporter>>,
}

impl Default for ImporterRegistry {
    fn default() -> Self {
        ImporterRegistry::new()
    }
}

impl ImporterRegistry {
    pub fn new() -> Self {
        Self {
            importers: Vec::new(),
        }
    }

    pub fn register_importer(&mut self, importer: Box<dyn StatementImporter>) {
        self.importers.push(importer);
    }

    pub fn importer(&self, name: &str) -> Option<&dyn StatementImporter> {
        self.importers
            .iter()
            .find(|i| i.importer_name() == name)
            .map(|v| &**v)
    }

    async fn maybe_create_session(
        db: &Database,
        existing_session: &Option<&mut ClientSession>,
    ) -> Result<Option<ClientSession>> {
        // If the caller already has a session, we don't need to create a new one.
        if existing_session.is_some() {
            Ok(None)
        } else {
            // If the caller doesn't have a session, we need to create one and start a transaction.
            info!("Creating new session for import because none was provided");
            let client = db.client();
            let mut session = client.start_session().await?;
            session.start_transaction().await?;
            Ok(Some(session))
        }
    }

    async fn import_with_importers(
        &self,
        importers: Vec<&dyn StatementImporter>,
        content: &str,
        db: &Database,
        session: Option<&mut ClientSession>,
        source_id: ObjectId,
    ) -> Result<()> {
        let mut new_session = Self::maybe_create_session(db, &session).await?;
        let effective_session = match &mut new_session {
            Some(s) => Some(s),
            None => session,
        };

        for importer in importers {
            if importer.content_matches(content).await == PathMatch::Match {
                // Run the importer.
                let result = importer
                    .import(content, db, effective_session, source_id)
                    .await;

                if new_session.is_some() {
                    // We created this session, so this is the complete unit of work.
                    // If we succeeded, commit the session; otherwise, abort it.
                    if result.is_ok() {
                        new_session.unwrap().commit_transaction().await?;
                    } else {
                        new_session.unwrap().abort_transaction().await?;
                    }
                }

                return result;
            }
        }
        Err(anyhow::anyhow!("No matching importer found"))
    }

    pub async fn import_statement_content(
        &self,
        content: &str,
        db: &Database,
        session: Option<&mut ClientSession>,
        source_id: ObjectId,
    ) -> Result<()> {
        let importers = self
            .importers
            .iter()
            .map(|i| i.as_ref())
            .collect::<Vec<&dyn StatementImporter>>();
        self.import_with_importers(importers, content, db, session, source_id)
            .await
    }

    pub async fn import_statement_files(
        &self,
        db: &Database,
        _session: Option<&mut ClientSession>,
        paths: Vec<PathBuf>,
    ) -> Result<()> {
        for path in paths {
            // Construct a new statement source ID for each file.
            let source_id = ObjectId::new();

            info!("attempting to import brokerage statement file: {:?}", path);

            // Find the set of importers that can handle this path.
            let mut viable_importers = Vec::<&dyn StatementImporter>::new();
            for importer in self.importers.iter() {
                if importer.path_may_match(&path).await == PathMatch::Match {
                    viable_importers.push(importer.as_ref());
                }
            }

            if viable_importers.is_empty() {
                info!(
                    "No viable importer found for file: {:?} based on filename",
                    path
                );
                continue;
            }

            // Read the file contents.
            let content = fs::read_to_string(&path)?;

            // Import the file contents using the first hard-match importer based on content.
            self.import_with_importers(viable_importers, &content, db, None, source_id)
                .await?;
        }
        Ok(())
    }
}
