use anyhow::Result;
use async_trait::async_trait;
use ibkr_flex_statement::Parser;
use mongodb::{ClientSession, Database, bson::oid::ObjectId};
use std::path::Path;
use tracing::info;

use crate::{path_match::PathMatch, statement_importer::StatementImporter, writers};

pub const IBKR_BROKERAGE_ID: &str = "ibkr";

pub struct IbkrFlexStatementImporter {}

impl Default for IbkrFlexStatementImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl IbkrFlexStatementImporter {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl StatementImporter for IbkrFlexStatementImporter {
    fn importer_name(&self) -> &'static str {
        "ibkr-flex"
    }

    async fn path_may_match(&self, path: &Path) -> PathMatch {
        if path.extension().is_some_and(|ext| ext == "xml") {
            PathMatch::Match
        } else {
            PathMatch::NoMatch
        }
    }

    async fn content_matches(&self, content: &str) -> PathMatch {
        if content.starts_with("<FlexQueryResponse") {
            PathMatch::Match
        } else {
            PathMatch::NoMatch
        }
    }

    async fn import(
        &self,
        content: &str,
        db: &Database,
        _session: Option<&mut ClientSession>,
        source_id: ObjectId,
    ) -> Result<()> {
        tracing::debug!(
            "Importing IBKR Flex with importer {}, source_id {}, string content {}",
            self.importer_name(),
            source_id,
            content
        );

        // Parse the IBKR Flex query content.
        let parser = Parser::new()?;
        let flex_statements = parser.parse_flex_query_response(content)?;

        let client = db.client();
        let mut session = client.start_session().await?;

        // Add each flex statement content to the database.
        for flex_statement in flex_statements {
            let brokerage_account = writers::maybe_add_brokerage_account(
                db,
                Some(&mut session),
                IBKR_BROKERAGE_ID,
                &flex_statement.account_info.account_id,
            )
            .await?;

            info!(
                "Parsed IBKR Flex statement for IBKR brokerage account: {}",
                brokerage_account.get_account_id(),
            );
        }

        Ok(())
    }
}
