use std::sync::Arc;

use anyhow::Result;
use brokerage_db::{
    account::BrokerageAccount,
    security::{Security, SecurityType},
};
use mongodb::{ClientSession, Database};
use tokio::sync::Mutex;
use tracing::{debug, info};

pub async fn maybe_add_brokerage_account(
    db: &Database,
    session: Option<Arc<Mutex<ClientSession>>>,
    brokerage_id: &str,
    account_id: &str,
) -> Result<BrokerageAccount> {
    let brokerage_account =
        BrokerageAccount::find_by_brokerage_and_account_id(db, brokerage_id, account_id).await?;
    if brokerage_account.is_none() {
        let new_account = BrokerageAccount::new(brokerage_id, account_id);
        new_account.insert(db, session).await?;
        info!(
            "Added new brokerage account: {} at {}",
            account_id, brokerage_id
        );
        Ok(new_account)
    } else {
        debug!(
            "Brokerage account already exists: {} at {}",
            account_id, brokerage_id
        );
        Ok(brokerage_account.unwrap())
    }
}

pub async fn maybe_add_security(
    db: &Database,
    session: Option<Arc<Mutex<ClientSession>>>,
    ticker: &str,
    listing_exchange: &str,
    ibkr_conid: Option<u32>,
) -> Result<Security> {
    let security = Security::find_by_ticker_and_exchange(db, ticker, listing_exchange).await?;
    if security.is_none() {
        let new_security = Security::new(SecurityType::Stock, ticker, listing_exchange, ibkr_conid);
        new_security.insert(db, session).await?;
        info!("Added security: {} on {}", ticker, listing_exchange);
        Ok(new_security)
    } else {
        debug!(
            "security already exists ({} at {}), skipping db insert",
            ticker, listing_exchange
        );
        Ok(security.unwrap())
    }
}
