use anyhow::Result;
use brokerage_db::account::BrokerageAccount;
use mongodb::{ClientSession, Database};
use tracing::{debug, info};

pub async fn maybe_add_brokerage_account(
    db: &Database,
    session: Option<&mut ClientSession>,
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
