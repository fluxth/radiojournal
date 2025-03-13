use std::sync::Arc;

use anyhow::Result;
use aws_config::{BehaviorVersion, meta::region::RegionProviderChain};
use aws_sdk_dynamodb::Client;

use crate::crud::Context;

const LOCALSTACK_ENDPOINT: &str = "http://localhost:4566";

/// If LOCALSTACK environment variable is true, use LocalStack endpoints.
/// You can use your own method for determining whether to use LocalStack endpoints.
pub fn use_localstack() -> bool {
    std::env::var("LOCALSTACK").unwrap_or_default() == "true"
}

pub async fn initialize() -> Result<Arc<Context>> {
    let region_provider = RegionProviderChain::default_provider().or_else("ap-southeast-1");

    let mut config = aws_config::defaults(BehaviorVersion::latest()).region(region_provider);
    if use_localstack() {
        config = config.endpoint_url(LOCALSTACK_ENDPOINT);
    };

    let config = config.load().await;

    let db_client = Client::new(&config);
    let table_name = std::env::var("DB_TABLE_NAME").expect("env DB_TABLE_NAME to be set");

    let context = Arc::new(Context::new(db_client, table_name));

    Ok(context)
}
