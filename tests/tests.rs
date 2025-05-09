mod fixtures;

use std::path::PathBuf;

use crate::ibkr_flex_statement_importer::IBKR_BROKERAGE_ID;
use anyhow::Result;
use brokerage_db::{
    account::BrokerageAccount, security::Security, trade_execution::TradeExecution,
};
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

    // Commit transactions.
    // db_desc.session.lock().await.commit_transaction().await?;

    // Verify the account was added.
    let brokerage_account = BrokerageAccount::find_by_brokerage_and_account_id(
        &db_desc.db,
        IBKR_BROKERAGE_ID,
        IBKR_ACCOUNT_ID,
    )
    .await?;
    assert!(brokerage_account.is_some());

    let brokerage_account = brokerage_account.unwrap();
    assert_eq!(brokerage_account.account_id(), fixtures::IBKR_ACCOUNT_ID);
    assert_eq!(brokerage_account.brokerage_id(), IBKR_BROKERAGE_ID);

    // Verify security was added.
    let securities_result =
        Security::find_by_ticker(&db_desc.db, fixtures::IBKR_SINGLE_TRADE_TICKER).await;
    assert!(securities_result.is_ok());

    let securities = securities_result.unwrap();
    assert_eq!(securities.len(), 1);
    assert_eq!(securities[0].ticker(), fixtures::IBKR_SINGLE_TRADE_TICKER);

    // Verify trade execution was added.
    let trade_execution = TradeExecution::find_by_brokerage_execution_id(
        &db_desc.db,
        IBKR_SINGLE_TRADE_BROKERAGE_EXECUTION_ID,
    )
    .await?;
    assert!(trade_execution.is_some());
    let trade_execution = trade_execution.unwrap();
    assert_eq!(
        trade_execution.brokerage_execution_id(),
        IBKR_SINGLE_TRADE_BROKERAGE_EXECUTION_ID
    );
    assert_eq!(trade_execution.security_id(), securities[0].id());

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

    // Commit transactions.
    // db_desc.session.lock().await.commit_transaction().await?;

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
    assert_eq!(brokerage_account.account_id(), fixtures::IBKR_ACCOUNT_ID);
    assert_eq!(brokerage_account.brokerage_id(), IBKR_BROKERAGE_ID);

    // Verify security was added.
    let securities_result =
        Security::find_by_ticker(&db_desc.db, fixtures::IBKR_SINGLE_TRADE_TICKER).await;
    assert!(securities_result.is_ok());

    let securities = securities_result.unwrap();
    assert_eq!(securities.len(), 1);
    assert_eq!(securities[0].ticker(), fixtures::IBKR_SINGLE_TRADE_TICKER);

    // Verify trade execution was added.
    let trade_execution = TradeExecution::find_by_brokerage_execution_id(
        &db_desc.db,
        IBKR_SINGLE_TRADE_BROKERAGE_EXECUTION_ID,
    )
    .await?;
    assert!(trade_execution.is_some());
    let trade_execution = trade_execution.unwrap();
    assert_eq!(
        trade_execution.brokerage_execution_id(),
        IBKR_SINGLE_TRADE_BROKERAGE_EXECUTION_ID
    );
    assert_eq!(trade_execution.security_id(), securities[0].id());

    Ok(())
}
