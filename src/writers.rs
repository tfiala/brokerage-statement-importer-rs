use std::sync::Arc;

use anyhow::Result;
use brokerage_db::{
    account::BrokerageAccount,
    security::{Security, SecurityType},
    trade_execution::{TradeExecution, TradeSide},
};
use mongodb::{ClientSession, Database, bson::oid::ObjectId};
use tokio::sync::Mutex;
use tracing::{debug, info};

pub struct TradeWriter {
    brokerage_account_id: Option<ObjectId>,
    brokerage_execution_id: Option<String>,
    commission: Option<f64>,
    execution_timestamp_ms: Option<i64>,
    quantity: Option<f64>,
    price: Option<f64>,
    security_id: Option<ObjectId>,
    side: Option<TradeSide>,
}

impl TradeWriter {
    pub fn new() -> Self {
        Self {
            brokerage_account_id: None,
            brokerage_execution_id: None,
            commission: None,
            execution_timestamp_ms: None,
            quantity: None,
            price: None,
            security_id: None,
            side: None,
        }
    }

    pub fn brokerage_account_id(mut self, id: ObjectId) -> Self {
        self.brokerage_account_id = Some(id);
        self
    }

    pub fn brokerage_execution_id(mut self, id: &str) -> Self {
        self.brokerage_execution_id = Some(id.to_owned());
        self
    }

    pub fn commission(mut self, commission: f64) -> Self {
        self.commission = Some(commission);
        self
    }

    pub fn execution_timestamp_ms(mut self, timestamp: i64) -> Self {
        self.execution_timestamp_ms = Some(timestamp);
        self
    }

    pub fn quantity(mut self, quantity: f64) -> Self {
        self.quantity = Some(quantity);
        self
    }

    pub fn price(mut self, price: f64) -> Self {
        self.price = Some(price);
        self
    }

    pub fn security_id(mut self, id: ObjectId) -> Self {
        self.security_id = Some(id);
        self
    }

    pub fn side(mut self, side: TradeSide) -> Self {
        self.side = Some(side);
        self
    }

    pub async fn insert(
        self,
        db: &Database,
        session: Option<Arc<Mutex<ClientSession>>>,
    ) -> Result<()> {
        let trade = TradeExecution::builder()
            .brokerage_account_id(self.brokerage_account_id.unwrap())
            .brokerage_execution_id(&self.brokerage_execution_id.unwrap())
            .commission(self.commission.unwrap())
            .execution_timestamp_ms(self.execution_timestamp_ms.unwrap())
            .quantity(self.quantity.unwrap())
            .price(self.price.unwrap())
            .security_id(self.security_id.unwrap())
            .side(self.side.unwrap())
            .build()?;

        trade.insert(db, session).await
    }
}

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
