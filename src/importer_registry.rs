use std::{fs, path::PathBuf, sync::Arc};

use crate::path_match::PathMatch;
use crate::statement_importer::StatementImporter;
use anyhow::Result;
use mongodb::{ClientSession, Database, bson::oid::ObjectId};
use tokio::sync::Mutex;
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

    async fn import_with_importers(
        &self,
        importers: Vec<&dyn StatementImporter>,
        content: &str,
        db: &Database,
        session: Option<Arc<Mutex<ClientSession>>>,
        source_id: ObjectId,
    ) -> Result<()> {
        for importer in importers {
            if importer.content_matches(content).await == PathMatch::Match {
                // Run the importer.
                let result = importer
                    .import(content, db, session.clone(), source_id)
                    .await;
                return result;
            }
        }
        Err(anyhow::anyhow!("No matching importer found"))
    }

    pub async fn import_statement_content(
        &self,
        content: &str,
        db: &Database,
        session: Option<Arc<Mutex<ClientSession>>>,
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
        session: Option<Arc<Mutex<ClientSession>>>,
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
            self.import_with_importers(viable_importers, &content, db, session.clone(), source_id)
                .await?;
        }
        Ok(())
    }
}
