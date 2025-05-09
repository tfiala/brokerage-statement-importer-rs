mod fixtures;

use std::path::PathBuf;

use crate::ibkr_flex_statement_importer::IBKR_BROKERAGE_ID;
use anyhow::Result;
use brokerage_db::account::BrokerageAccount;
use brokerage_statement_importer::{importer_registry::ImporterRegistry, *};
use fixtures::*;
use mongodb::bson::oid::ObjectId;
use rstest::rstest;
use tracing_test::traced_test;

#[test]
fn test_filter_unimported_files_with_no_previous_imports() {
    let paths = vec![
        PathBuf::from("test1.xml"),
        PathBuf::from("test2.xml"),
        PathBuf::from("test3.xml"),
    ];
    let filtered_paths = filter_unimported_files(paths).unwrap();
    assert_eq!(filtered_paths.len(), 3);
}

#[rstest]
#[awt]
#[traced_test]
#[tokio::test]
async fn mongodb_sessions_work(#[future] db_desc: Result<DbDesc>) -> Result<()> {
    let db_desc = db_desc?;

    // Create a session.
    let mut session = db_desc.db.client().start_session().await?;

    // Start a transaction.
    session.start_transaction().await?;

    // Commit the session.
    session.commit_transaction().await?;

    Ok(())
}

#[rstest]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_import_string_ibkr_single_trade_execution(
    #[future] db_desc: Result<DbDesc>,
    registry: ImporterRegistry,
    single_trade_flex: &str,
) -> Result<()> {
    let db_desc = db_desc?;

    // Test the importer
    let source_id = ObjectId::new();

    registry
        .import_statement_content(single_trade_flex, &db_desc.db, None, source_id)
        .await?;

    // Verify the account was added.
    let brokerage_account = BrokerageAccount::find_by_brokerage_and_account_id(
        &db_desc.db,
        IBKR_BROKERAGE_ID,
        IBKR_ACCOUNT_ID,
    )
    .await?;
    assert!(brokerage_account.is_some());

    let brokerage_account = brokerage_account.unwrap();
    assert_eq!(
        brokerage_account.get_account_id(),
        fixtures::IBKR_ACCOUNT_ID
    );
    assert_eq!(brokerage_account.get_brokerage_id(), IBKR_BROKERAGE_ID);

    Ok(())
}

#[rstest]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_import_file_ibkr_single_trade_execution(
    #[future] db_desc: Result<DbDesc>,
    registry: ImporterRegistry,
    single_trade_flex_pathbuf: PathBuf,
) -> Result<()> {
    let db_desc = db_desc?;

    // Test the importer
    registry
        .import_statement_files(&db_desc.db, None, vec![single_trade_flex_pathbuf])
        .await?;

    // Verify the account was added.
    let brokerage_account = BrokerageAccount::find_by_brokerage_and_account_id(
        &db_desc.db,
        IBKR_BROKERAGE_ID,
        fixtures::IBKR_ACCOUNT_ID,
    )
    .await?;
    assert!(
        brokerage_account.is_some(),
        "Brokerage account should exist"
    );

    let brokerage_account = brokerage_account.unwrap();
    assert_eq!(
        brokerage_account.get_account_id(),
        fixtures::IBKR_ACCOUNT_ID
    );
    assert_eq!(brokerage_account.get_brokerage_id(), IBKR_BROKERAGE_ID);

    Ok(())
}
