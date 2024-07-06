pub mod models;

use std::sync::Arc;

use anyhow::Result;
use aws_sdk_dynamodb::types::{AttributeValue, Select};

use crate::crud::Context;
use models::{StationId, StationInDB, StationInDBCreate};

pub struct CRUDStation {
    context: Arc<Context>,
}

impl CRUDStation {
    pub fn new(context: Arc<Context>) -> Self {
        Self { context }
    }

    pub async fn get_station(&self, station_id: StationId) -> Result<Option<StationInDB>> {
        let resp = self
            .context
            .db_client
            .get_item()
            .table_name(&self.context.db_table)
            .key("pk", AttributeValue::S(StationInDB::get_pk()))
            .key("sk", AttributeValue::S(StationInDB::get_sk(station_id)))
            .send()
            .await?;

        if let Some(item) = resp.item {
            Ok(Some(serde_dynamo::from_item(item)?))
        } else {
            Ok(None)
        }
    }

    pub async fn list_stations(&self, limit: i32) -> Result<Vec<StationInDB>> {
        let resp = self
            .context
            .db_client
            .query()
            .table_name(&self.context.db_table)
            .key_condition_expression("pk = :pk AND begins_with(sk, :sk)")
            .expression_attribute_values(":pk", AttributeValue::S(StationInDB::get_pk()))
            .expression_attribute_values(":sk", AttributeValue::S(StationInDB::get_sk_prefix()))
            .select(Select::AllAttributes)
            .limit(limit)
            .send()
            .await?;

        let items = resp.items().to_vec();
        let stations: Vec<StationInDB> = serde_dynamo::from_items(items)?;

        Ok(stations)
    }

    pub async fn create_station(&self, station_create: StationInDBCreate) -> Result<StationInDB> {
        let station_internal: StationInDB = station_create.into();

        self.context
            .db_client
            .put_item()
            .table_name(&self.context.db_table)
            .set_item(Some(serde_dynamo::to_item(station_internal.clone())?))
            .send()
            .await?;

        Ok(station_internal)
    }
}
