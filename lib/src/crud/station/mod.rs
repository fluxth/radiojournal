pub mod models;
mod provider;

use std::sync::Arc;

use anyhow::Result;

use crate::crud::Context;
use models::{StationId, StationInDB, StationInDBCreate};
use provider::{DynamoDBProvider, GetItemInput, PutItemInput, QueryPrefixConfig, QueryPrefixInput};

pub struct CRUDStation {
    provider: DynamoDBProvider,
}

impl CRUDStation {
    pub fn new(context: Arc<Context>) -> Self {
        Self {
            provider: DynamoDBProvider::new(context),
        }
    }

    pub async fn get_station(&self, station_id: StationId) -> Result<Option<StationInDB>> {
        let resp = self
            .provider
            .get_item(GetItemInput {
                pk: StationInDB::get_pk(),
                sk: StationInDB::get_sk(station_id),
            })
            .await?;

        if let Some(item) = resp.item {
            Ok(Some(serde_dynamo::from_item(item)?))
        } else {
            Ok(None)
        }
    }

    pub async fn list_stations(&self, limit: i32) -> Result<Vec<StationInDB>> {
        let resp = self
            .provider
            .query_prefix(
                QueryPrefixInput {
                    pk: StationInDB::get_pk(),
                    sk_prefix: StationInDB::get_sk_prefix(),
                },
                QueryPrefixConfig { limit },
            )
            .await?;

        let items = resp.items().to_vec();
        let stations: Vec<StationInDB> = serde_dynamo::from_items(items)?;

        Ok(stations)
    }

    pub async fn create_station(&self, station_create: StationInDBCreate) -> Result<StationInDB> {
        let station: StationInDB = station_create.into();

        self.provider
            .put_item(PutItemInput {
                item: serde_dynamo::to_item(station.clone())?,
            })
            .await?;

        Ok(station)
    }
}
