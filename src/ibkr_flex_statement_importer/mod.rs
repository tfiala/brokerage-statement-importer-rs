use anyhow::Result;
use async_trait::async_trait;
use ibkr_flex_statement::{Parser, Statement};
use mongodb::{ClientSession, Database, bson::oid::ObjectId};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::Arc,
};
use tokio::sync::Mutex;
use tracing::info;

use crate::{
    path_match::PathMatch,
    statement_importer::StatementImporter,
    writers::{self, TradeWriter},
};

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

    async fn import_securities(
        &self,
        statement: &Statement,
        db: &Database,
        session: Option<Arc<Mutex<ClientSession>>>,
    ) -> Result<HashMap<u32, ObjectId>> {
        let mut conid_map = HashMap::<u32, ObjectId>::new();

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

            conid_map.insert(conid, security.id());

            info!(
                "Parsed IBKR Flex statement for security: {}",
                security.ticker()
            );
        }

        Ok(conid_map)
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
            "Importing IBKR Flex statement for brokerage account: {}",
            brokerage_account.account_id(),
        );

        let conid_security_map = self
            .import_securities(statement, db, session.clone())
            .await?;

        // Import the trades.
        for trade in &statement.trades {
            let security_id = conid_security_map
                .get(&trade.conid)
                .ok_or_else(|| anyhow::anyhow!("Security not found for conid {}", trade.conid))?;

            let trade_side = match trade.side {
                ibkr_flex_statement::trade::TradeSide::Buy => {
                    brokerage_db::trade_execution::TradeSide::Buy
                }
                ibkr_flex_statement::trade::TradeSide::Sell => {
                    brokerage_db::trade_execution::TradeSide::Sell
                }
            };

            TradeWriter::new()
                .brokerage_account_id(brokerage_account.id())
                .brokerage_execution_id(&trade.execution_id)
                .commission(trade.commission)
                .execution_timestamp_ms(trade.execution_timestamp_ms)
                .quantity(trade.quantity)
                .price(trade.price)
                .security_id(*security_id)
                .side(trade_side)
                .insert(db, session.clone())
                .await?;
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

        // Add each flex statement content to the database.
        for flex_statement in flex_statements {
            self.import_flex_statement(&flex_statement, db, session.clone(), source_id)
                .await?;
        }

        Ok(())
    }
}
