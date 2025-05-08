use anyhow::Result;
use brokerage_db::account::BrokerageAccount;
use ibkr_flex_statement::Parser;
use mongodb::{ClientSession, Database};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};

pub const IBKR_BROKERAGE_ID: &str = "ibkr";

async fn maybe_add_brokerage_account(
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

pub fn filter_unimported_files(paths: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    Ok(paths.clone())
}

pub async fn import_ibkr_flex_content(
    db: &Database,
    short_filename: &str,
    contents: &str,
) -> Result<()> {
    tracing::info!("Importing IBKR Flex content from file: {}", short_filename);

    // Parse the IBKR Flex query content.
    let parser = Parser::new()?;
    let flex_statements = parser.parse_flex_query_response(contents)?;

    let client = db.client();
    let mut session = client.start_session().await?;

    // Add each flex statement content to the database.
    for flex_statement in flex_statements {
        let brokerage_account = maybe_add_brokerage_account(
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

pub async fn import_ibkr_flex_statement_files(db: &Database, paths: Vec<PathBuf>) -> Result<()> {
    for path in paths {
        println!("Importing file: {:?}", path);
        let short_filename = path.file_name().unwrap();
        let contents = fs::read_to_string(&path)?;
        import_ibkr_flex_content(db, short_filename.to_str().unwrap(), &contents).await?;
    }
    Ok(())
}
