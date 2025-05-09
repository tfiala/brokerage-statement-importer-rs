use anyhow::Result;
use async_trait::async_trait;
use ibkr_flex_statement::{Parser, Statement};
use mongodb::{ClientSession, Database, bson::oid::ObjectId};
use std::{collections::HashSet, path::Path, sync::Arc};
use tokio::sync::Mutex;
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

    async fn import_flex_statement(
        &self,
        statement: &Statement,
        db: &Database,
        session: Option<Arc<Mutex<ClientSession>>>,
        _source_id: ObjectId,
    ) -> Result<()> {
        let brokerage_account = writers::maybe_add_brokerage_account(
            db,
            session.clone(),
            IBKR_BROKERAGE_ID,
            &statement.account_info.account_id,
        )
        .await?;

        info!(
            "Parsed IBKR Flex statement for IBKR brokerage account: {}",
            brokerage_account.account_id(),
        );

        let securities = statement
            .trades
            .iter()
            .map(|trade| {
                (
                    trade.ticker.clone(),
                    trade.listing_exchange.clone(),
                    trade.conid,
                )
            })
            .collect::<HashSet<(String, String, u32)>>();

        for (ticker, listing_exchange, conid) in securities {
            let security = writers::maybe_add_security(
                db,
                session.clone(),
                &ticker,
                &listing_exchange,
                Some(conid),
            )
            .await?;

            info!(
                "Parsed IBKR Flex statement for security: {}",
                security.ticker()
            );
        }

        Ok(())
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
        session: Option<Arc<Mutex<ClientSession>>>,
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

        // let client = db.client();
        // let mut session = client.start_session().await?;

        // Add each flex statement content to the database.
        for flex_statement in flex_statements {
            self.import_flex_statement(&flex_statement, db, session.clone(), source_id)
                .await?;
        }

        Ok(())
    }
}
